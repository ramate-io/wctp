// =================================================================================================
// GPU MARCHING CUBES — TYPE SAFE PIPELINE
// Bevy 0.17 — fully self-contained and overwrite-safe
// compute() returns immutable CPU-side mesh data
// =================================================================================================

use bevy::{
	prelude::*,
	render::{
		render_asset::RenderAssetUsages,
		render_resource::*,
		renderer::{RenderDevice, RenderQueue},
		view::Visibility,
	},
};
use bytemuck::{Pod, Zeroable};

// =================================================================================================
// GPU-REPRESENTABLE STRUCTS
// =================================================================================================

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Bounds {
	pub enabled: u32,
	pub min: Vec2,
	pub max: Vec2,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Sampling3D {
	pub chunk_origin: Vec3,
	pub chunk_size: Vec3,
	pub resolution: UVec3,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TerrainConfigGpu {
	pub seed: u32,
	pub base_resolution: u32,
	pub height_scale: f32,
	pub use_volumetric: u32,
}

impl From<&crate::terrain::TerrainConfig> for TerrainConfigGpu {
	fn from(c: &crate::terrain::TerrainConfig) -> Self {
		Self {
			seed: c.seed,
			base_resolution: c.base_resolution as u32,
			height_scale: c.height_scale,
			use_volumetric: c.use_volumetric as u32,
		}
	}
}

// =================================================================================================
// CPU-SIDE RESULT (RETURNED BY compute())
// =================================================================================================

pub struct GpuMeshData {
	pub positions: Vec<[f32; 3]>,
	pub normals: Vec<[f32; 3]>,
	pub uvs: Vec<[f32; 2]>,
}

// =================================================================================================
// BUFFER HELPERS
// =================================================================================================

fn new_storage(device: &RenderDevice, size_bytes: usize) -> Buffer {
	device.create_buffer(&BufferDescriptor {
		label: None,
		size: size_bytes as u64,
		usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
		mapped_at_creation: false,
	})
}

fn new_uniform<T: Pod>(device: &RenderDevice, v: &T) -> Buffer {
	device.create_buffer_with_data(&BufferInitDescriptor {
		label: None,
		contents: bytemuck::bytes_of(v),
		usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
	})
}

fn read_vec<T: Pod>(
	device: &RenderDevice,
	queue: &RenderQueue,
	src: &Buffer,
	count: usize,
) -> Vec<T> {
	let size = (count * std::mem::size_of::<T>()) as u64;

	let staging = device.create_buffer(&BufferDescriptor {
		label: None,
		size,
		usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
		mapped_at_creation: false,
	});

	let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
	encoder.copy_buffer_to_buffer(src, 0, &staging, 0, size);
	queue.submit(Some(encoder.finish()));

	let slice = staging.slice(..);
	slice.map_async(MapMode::Read, |_| {});
	device.poll(Maintain::Wait);

	let range = slice.get_mapped_range();
	let result: Vec<T> = bytemuck::cast_slice(&range).to_vec();
	drop(range);
	staging.unmap();

	result
}

fn read_u32(device: &RenderDevice, queue: &RenderQueue, src: &Buffer, idx: u32) -> u32 {
	let staging = device.create_buffer(&BufferDescriptor {
		label: None,
		size: 4,
		usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
		mapped_at_creation: false,
	});

	let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
	encoder.copy_buffer_to_buffer(src, (idx * 4) as u64, &staging, 0, 4);
	queue.submit(Some(encoder.finish()));

	let slice = staging.slice(..);
	slice.map_async(MapMode::Read, |_| {});
	device.poll(Maintain::Wait);

	let range = slice.get_mapped_range();
	let v = u32::from_le_bytes(range[0..4].try_into().unwrap());
	drop(range);
	staging.unmap();
	v
}

// =================================================================================================
// PIPELINE CREATION
// =================================================================================================

fn load_pipeline(
	pipeline_cache: &mut PipelineCache,
	shader: Handle<Shader>,
	entry: &'static str,
	layout: &BindGroupLayout,
) -> ComputePipeline {
	use std::borrow::Cow;

	let pipeline_descriptor = ComputePipelineDescriptor {
		label: Some(Cow::Owned(format!("compute_pipeline_{}", entry))),
		layout: vec![layout.clone()],
		shader,
		shader_defs: vec![],
		entry_point: Cow::Borrowed(entry),
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	};

	// Queue the pipeline for compilation
	let pipeline_id = pipeline_cache.queue_compute_pipeline(pipeline_descriptor);

	// Process the pipeline cache until the pipeline is ready
	loop {
		pipeline_cache.process_queue();
		if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) {
			return pipeline.clone();
		}
		std::thread::yield_now();
	}
}

// =================================================================================================
// TYPESAFE OVERWRITE-PROOF PIPELINE
// =================================================================================================

pub struct GpuMarchingCubesPipeline {
	sampling: Sampling3D,
	terrain_cfg: TerrainConfigGpu,
	bounds: Bounds,
	seed: i32,

	dispatch: UVec3,
	voxel_count: u32,
	block_count: u32,

	classify_pipeline: ComputePipeline,
	prefix_local_pipeline: ComputePipeline,
	prefix_block_pipeline: ComputePipeline,
	prefix_add_pipeline: ComputePipeline,
	mesh_pipeline: ComputePipeline,

	layout: BindGroupLayout,
}

impl GpuMarchingCubesPipeline {
	pub fn new(
		device: &RenderDevice,
		pipeline_cache: &mut PipelineCache,
		shaders: &mut Assets<Shader>,
		sampling: Sampling3D,
		terrain_cfg_src: &crate::terrain::TerrainConfig,
		bounds: Bounds,
		seed: i32,
	) -> Self {
		let voxel_count = sampling.resolution.x * sampling.resolution.y * sampling.resolution.z;
		let block_count = (voxel_count + 255) / 256;

		let dispatch = UVec3::new(
			(sampling.resolution.x + 7) / 8,
			(sampling.resolution.y + 7) / 8,
			(sampling.resolution.z + 7) / 8,
		);

		// --- Shared layout (all shaders use same BG layout) ---
		let layout_entries = &[BindGroupLayoutEntry {
			binding: 0,
			visibility: ShaderStages::COMPUTE,
			ty: BindingType::Buffer {
				ty: BufferBindingType::Storage { read_only: false },
				has_dynamic_offset: false,
				min_binding_size: None,
			},
			count: None,
		}];
		let layout =
			device.create_bind_group_layout(Some("compute_bind_group_layout"), layout_entries);

		// Load shaders through Bevy's asset system
		let classify_shader = shaders.add(Shader::from_wgsl(
			include_str!("../assets/proc/classify_voxels.wgsl"),
			"classify_voxels.wgsl",
		));
		let prefix_local_shader = shaders.add(Shader::from_wgsl(
			include_str!("../assets/proc/prefix_scan_local.wgsl"),
			"prefix_scan_local.wgsl",
		));
		let prefix_block_shader = shaders.add(Shader::from_wgsl(
			include_str!("../assets/proc/block_prefix.wgsl"),
			"block_prefix.wgsl",
		));
		let prefix_add_shader = shaders.add(Shader::from_wgsl(
			include_str!("../assets/proc/block_sum.wgsl"),
			"block_sum.wgsl",
		));
		let mesh_shader = shaders.add(Shader::from_wgsl(
			include_str!("../assets/proc/compute_mesh.wgsl"),
			"compute_mesh.wgsl",
		));

		let classify_pipeline = load_pipeline(pipeline_cache, classify_shader, "main", &layout);
		let prefix_local_pipeline =
			load_pipeline(pipeline_cache, prefix_local_shader, "main", &layout);
		let prefix_block_pipeline =
			load_pipeline(pipeline_cache, prefix_block_shader, "main", &layout);
		let prefix_add_pipeline = load_pipeline(pipeline_cache, prefix_add_shader, "main", &layout);
		let mesh_pipeline = load_pipeline(pipeline_cache, mesh_shader, "compute_mesh", &layout);

		Self {
			sampling,
			terrain_cfg: TerrainConfigGpu::from(terrain_cfg_src),
			bounds,
			seed,
			dispatch,
			voxel_count,
			block_count,
			classify_pipeline,
			prefix_local_pipeline,
			prefix_block_pipeline,
			prefix_add_pipeline,
			mesh_pipeline,
			layout,
		}
	}

	// =================================================================================================
	// TYPESAFE: compute() allocates fresh buffers, runs, reads back, and returns CPU mesh data
	// =================================================================================================
	pub fn compute(&self, device: &RenderDevice, queue: &RenderQueue) -> GpuMeshData {
		let voxel_count = self.voxel_count;
		let block_count = self.block_count;

		// Allocate new buffers — NEVER reused, cannot overwrite
		let cube_index = new_storage(device, voxel_count as usize * 4);
		let tri_counts = new_storage(device, voxel_count as usize * 4);
		let tri_offset = new_storage(device, voxel_count as usize * 4);
		let block_sums = new_storage(device, block_count as usize * 4);
		let block_prefix = new_storage(device, block_count as usize * 4);

		// Output buffers
		let max_verts = voxel_count * 15;
		let out_pos = new_storage(device, max_verts as usize * 12);
		let out_normals = new_storage(device, max_verts as usize * 12);
		let out_uvs = new_storage(device, max_verts as usize * 8);

		// Uniform packs
		let sampling_buf = new_uniform(device, &self.sampling);
		let cfg_buf = new_uniform(device, &self.terrain_cfg);
		let bounds_buf = new_uniform(device, &self.bounds);
		let seed_buf = new_uniform(device, &self.seed);

		// Build bind groups on demand for each pass
		// PASS 1: classify - needs cube_index and tri_counts
		let classify_bind_entries = &[BindGroupEntry {
			binding: 0,
			resource: BindingResource::Buffer(BufferBinding {
				buffer: &cube_index,
				offset: 0,
				size: None,
			}),
		}];
		let classify_bind = device.create_bind_group(
			Some("classify_bind_group"),
			&self.layout,
			classify_bind_entries,
		);

		// PASS 2: prefix_local - needs tri_counts, tri_offset, block_sums
		let prefix_local_bind_entries = &[BindGroupEntry {
			binding: 0,
			resource: BindingResource::Buffer(BufferBinding {
				buffer: &tri_counts,
				offset: 0,
				size: None,
			}),
		}];
		let prefix_local_bind = device.create_bind_group(
			Some("prefix_local_bind_group"),
			&self.layout,
			prefix_local_bind_entries,
		);

		// PASS 3: prefix_block - needs block_sums, block_prefix
		let prefix_block_bind_entries = &[BindGroupEntry {
			binding: 0,
			resource: BindingResource::Buffer(BufferBinding {
				buffer: &block_sums,
				offset: 0,
				size: None,
			}),
		}];
		let prefix_block_bind = device.create_bind_group(
			Some("prefix_block_bind_group"),
			&self.layout,
			prefix_block_bind_entries,
		);

		// PASS 4: prefix_add - needs tri_offset, block_prefix
		let prefix_add_bind_entries = &[BindGroupEntry {
			binding: 0,
			resource: BindingResource::Buffer(BufferBinding {
				buffer: &tri_offset,
				offset: 0,
				size: None,
			}),
		}];
		let prefix_add_bind = device.create_bind_group(
			Some("prefix_add_bind_group"),
			&self.layout,
			prefix_add_bind_entries,
		);

		// PASS 5: mesh - needs out_positions, out_normals, out_uvs
		let mesh_bind_entries = &[BindGroupEntry {
			binding: 0,
			resource: BindingResource::Buffer(BufferBinding {
				buffer: &out_pos,
				offset: 0,
				size: None,
			}),
		}];
		let mesh_bind =
			device.create_bind_group(Some("mesh_bind_group"), &self.layout, mesh_bind_entries);

		// ---------------- Run compute passes ----------------
		let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());

		// PASS 1: classify
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.classify_pipeline);
			pass.set_bind_group(0, &classify_bind, &[]);
			pass.dispatch_workgroups(self.dispatch.x, self.dispatch.y, self.dispatch.z);
		}

		// PASS 2: prefix_local
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.prefix_local_pipeline);
			pass.set_bind_group(0, &prefix_local_bind, &[]);
			pass.dispatch_workgroups(block_count, 1, 1);
		}

		// PASS 3: prefix_block
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.prefix_block_pipeline);
			pass.set_bind_group(0, &prefix_block_bind, &[]);
			pass.dispatch_workgroups(1, 1, 1);
		}

		// PASS 4: prefix_add
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.prefix_add_pipeline);
			pass.set_bind_group(0, &prefix_add_bind, &[]);
			pass.dispatch_workgroups(block_count, 1, 1);
		}

		// PASS 5: mesh
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.mesh_pipeline);
			pass.set_bind_group(0, &mesh_bind, &[]);
			pass.dispatch_workgroups(self.dispatch.x, self.dispatch.y, self.dispatch.z);
		}

		queue.submit(std::iter::once(encoder.finish()));

		// ---------------- Read back mesh ----------------
		let last = voxel_count - 1;
		let prefix = read_u32(device, queue, &tri_offset, last);
		let count = read_u32(device, queue, &tri_counts, last);
		let total_tris = prefix + count;
		let total_verts = total_tris * 3;

		let positions = read_vec::<[f32; 3]>(device, queue, &out_pos, total_verts as usize);
		let normals = read_vec::<[f32; 3]>(device, queue, &out_normals, total_verts as usize);
		let uvs = read_vec::<[f32; 2]>(device, queue, &out_uvs, total_verts as usize);

		GpuMeshData { positions, normals, uvs }
	}
}

// =================================================================================================
// BEVY MESH CONSTRUCTION
// =================================================================================================

pub fn spawn_mesh_from_gpu(
	commands: &mut Commands,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<StandardMaterial>,
	data: &GpuMeshData,
	origin: Vec3,
) {
	let mut mesh = Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
	);

	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.clone());
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.clone());
	mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs.clone());

	let mesh_handle = meshes.add(mesh);
	let material_handle = materials.add(StandardMaterial::from(Color::WHITE));

	commands.spawn((
		Mesh3d(mesh_handle),
		MeshMaterial3d::<StandardMaterial>(material_handle),
		Transform::from_translation(origin),
	));
}
