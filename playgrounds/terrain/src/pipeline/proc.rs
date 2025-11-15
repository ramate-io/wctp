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
	shaders: &mut Assets<Shader>,
	shader_src: &'static str,
	shader_path: &'static str,
	entry: &'static str,
	layout: &BindGroupLayout,
) -> ComputePipeline {
	use std::borrow::Cow;

	let shader = shaders.add(Shader::from_wgsl(shader_src, shader_path));
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

// Helper to create bind group layout entries
fn create_uniform_layout_entry(binding: u32) -> BindGroupLayoutEntry {
	BindGroupLayoutEntry {
		binding,
		visibility: ShaderStages::COMPUTE,
		ty: BindingType::Buffer {
			ty: BufferBindingType::Uniform,
			has_dynamic_offset: false,
			min_binding_size: None,
		},
		count: None,
	}
}

fn create_storage_layout_entry(binding: u32, read_only: bool) -> BindGroupLayoutEntry {
	BindGroupLayoutEntry {
		binding,
		visibility: ShaderStages::COMPUTE,
		ty: BindingType::Buffer {
			ty: BufferBindingType::Storage { read_only },
			has_dynamic_offset: false,
			min_binding_size: None,
		},
		count: None,
	}
}

// Helper to create bind group entries
fn create_buffer_entry(binding: u32, buffer: &Buffer) -> BindGroupEntry {
	BindGroupEntry {
		binding,
		resource: BindingResource::Buffer(BufferBinding { buffer, offset: 0, size: None }),
	}
}

// Helper to create a bind group from buffers
fn create_bind_group(
	device: &RenderDevice,
	label: &str,
	layout: &BindGroupLayout,
	buffers: &[&Buffer],
) -> BindGroup {
	let entries: Vec<BindGroupEntry> = buffers
		.iter()
		.enumerate()
		.map(|(i, buffer)| create_buffer_entry(i as u32, buffer))
		.collect();
	device.create_bind_group(Some(label), layout, &entries)
}

// =================================================================================================
// TYPESAFE OVERWRITE-PROOF PIPELINE
// =================================================================================================
// GpuMarchingCubesPipeline
// ============================================================================
//
// WGSL binding expectations (you should match these in your wgsl):
//
// classify_voxels.wgsl:
//   @group(0) @binding(0) var<uniform> sampling : Sampling3D;
//   @group(0) @binding(1) var<uniform> terrain_config : TerrainConfig;
//   @group(0) @binding(2) var<uniform> bounds : Bounds;
//   @group(0) @binding(3) var<uniform> seed : i32;
//   @group(0) @binding(4) var<storage, read_write> cube_index : array<u32>;
//   @group(0) @binding(5) var<storage, read_write> tri_counts : array<u32>;
//
// prefix_scan_local.wgsl:
//   @group(0) @binding(0) var<storage, read>       tri_counts : array<u32>;
//   @group(0) @binding(1) var<storage, read_write> tri_offset : array<u32>;
//   @group(0) @binding(2) var<storage, read_write> block_sums : array<u32>;
//
// block_prefix.wgsl:
//   @group(0) @binding(0) var<storage, read_write> block_sums : array<u32>;
//   @group(0) @binding(1) var<storage, read_write> block_prefix : array<u32>;
//
// block_sum.wgsl (add block prefix into tri_offset):
//   @group(0) @binding(0) var<storage, read_write> tri_offset : array<u32>;
//   @group(0) @binding(1) var<storage, read>       block_prefix : array<u32>;
//
// compute_mesh.wgsl (your version from earlier):
//   @group(0) @binding(0) var<uniform> sampling : Sampling3D;
//   @group(0) @binding(1) var<storage, read> cube_index : array<u32>;
//   @group(0) @binding(2) var<storage, read> tri_offset : array<u32>;
//   @group(0) @binding(3) var<storage, read_write> out_positions : array<vec3<f32>>;
//   @group(0) @binding(4) var<storage, read_write> out_normals   : array<vec3<f32>>;
//   @group(0) @binding(5) var<storage, read_write> out_uvs       : array<vec2<f32>>;
//   @group(0) @binding(6) var<uniform> terrain_config : TerrainConfig;
//   @group(0) @binding(7) var<uniform> bounds : Bounds;
//   @group(0) @binding(8) var<uniform> seed : i32;
// ============================================================================
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

	classify_layout: BindGroupLayout,
	prefix_local_layout: BindGroupLayout,
	prefix_block_layout: BindGroupLayout,
	prefix_add_layout: BindGroupLayout,
	mesh_layout: BindGroupLayout,
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

		// --- Layouts per pass ---

		let classify_layout = device.create_bind_group_layout(
			Some("mc_classify_layout"),
			&[
				create_uniform_layout_entry(0),        // sampling
				create_uniform_layout_entry(1),        // terrain_config
				create_uniform_layout_entry(2),        // bounds
				create_uniform_layout_entry(3),        // seed
				create_storage_layout_entry(4, false), // cube_index
				create_storage_layout_entry(5, false), // tri_counts
			],
		);

		let prefix_local_layout = device.create_bind_group_layout(
			Some("mc_prefix_local_layout"),
			&[
				create_storage_layout_entry(0, true),  // tri_counts (read)
				create_storage_layout_entry(1, false), // tri_offset (write)
				create_storage_layout_entry(2, false), // block_sums (write)
			],
		);

		let prefix_block_layout = device.create_bind_group_layout(
			Some("mc_prefix_block_layout"),
			&[
				create_storage_layout_entry(0, false), // block_sums
				create_storage_layout_entry(1, false), // block_prefix
			],
		);

		let prefix_add_layout = device.create_bind_group_layout(
			Some("mc_prefix_add_layout"),
			&[
				create_storage_layout_entry(0, false), // tri_offset
				create_storage_layout_entry(1, true),  // block_prefix (read)
			],
		);

		let mesh_layout = device.create_bind_group_layout(
			Some("mc_mesh_layout"),
			&[
				create_uniform_layout_entry(0),        // sampling
				create_storage_layout_entry(1, true),  // cube_index (read)
				create_storage_layout_entry(2, true),  // tri_offset (read)
				create_storage_layout_entry(3, false), // out_positions
				create_storage_layout_entry(4, false), // out_normals
				create_storage_layout_entry(5, false), // out_uvs
				create_uniform_layout_entry(6),        // terrain_config
				create_uniform_layout_entry(7),        // bounds
				create_uniform_layout_entry(8),        // seed
			],
		);

		// Pipelines
		let classify_pipeline = load_pipeline(
			pipeline_cache,
			shaders,
			include_str!("../assets/proc/classify_voxels.wgsl"),
			"classify_voxels.wgsl",
			"main",
			&classify_layout,
		);
		let prefix_local_pipeline = load_pipeline(
			pipeline_cache,
			shaders,
			include_str!("../assets/proc/prefix_scan_local.wgsl"),
			"prefix_scan_local.wgsl",
			"main",
			&prefix_local_layout,
		);
		let prefix_block_pipeline = load_pipeline(
			pipeline_cache,
			shaders,
			include_str!("../assets/proc/block_prefix.wgsl"),
			"block_prefix.wgsl",
			"main",
			&prefix_block_layout,
		);
		let prefix_add_pipeline = load_pipeline(
			pipeline_cache,
			shaders,
			include_str!("../assets/proc/block_sum.wgsl"),
			"block_sum.wgsl",
			"main",
			&prefix_add_layout,
		);
		let mesh_pipeline = load_pipeline(
			pipeline_cache,
			shaders,
			include_str!("../assets/proc/compute_mesh.wgsl"),
			"compute_mesh.wgsl",
			"compute_mesh",
			&mesh_layout,
		);

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
			classify_layout,
			prefix_local_layout,
			prefix_block_layout,
			prefix_add_layout,
			mesh_layout,
		}
	}

	pub fn compute(&self, device: &RenderDevice, queue: &RenderQueue) -> GpuMeshData {
		let voxel_count = self.voxel_count;
		let block_count = self.block_count;

		// Allocate fresh buffers each call (no overwrites)
		let cube_index = new_storage(device, voxel_count as usize * 4);
		let tri_counts = new_storage(device, voxel_count as usize * 4);
		let tri_offset = new_storage(device, voxel_count as usize * 4);
		let block_sums = new_storage(device, block_count as usize * 4);
		let block_prefix = new_storage(device, block_count as usize * 4);

		// Output (max: 15 verts per voxel)
		let max_verts = voxel_count * 15;
		let out_pos = new_storage(device, max_verts as usize * 12);
		let out_normals = new_storage(device, max_verts as usize * 12);
		let out_uvs = new_storage(device, max_verts as usize * 8);

		// Uniform buffers for this call
		let sampling_buf = new_uniform(device, &self.sampling);
		let cfg_buf = new_uniform(device, &self.terrain_cfg);
		let bounds_buf = new_uniform(device, &self.bounds);
		let seed_buf = new_uniform(device, &self.seed);

		// PASS 1: classify
		let classify_bind = create_bind_group(
			device,
			"mc_classify_bind",
			&self.classify_layout,
			&[&sampling_buf, &cfg_buf, &bounds_buf, &seed_buf, &cube_index, &tri_counts],
		);

		// PASS 2: prefix_local
		let prefix_local_bind = create_bind_group(
			device,
			"mc_prefix_local_bind",
			&self.prefix_local_layout,
			&[&tri_counts, &tri_offset, &block_sums],
		);

		// PASS 3: prefix_block
		let prefix_block_bind = create_bind_group(
			device,
			"mc_prefix_block_bind",
			&self.prefix_block_layout,
			&[&block_sums, &block_prefix],
		);

		// PASS 4: prefix_add
		let prefix_add_bind = create_bind_group(
			device,
			"mc_prefix_add_bind",
			&self.prefix_add_layout,
			&[&tri_offset, &block_prefix],
		);

		// PASS 5: mesh
		let mesh_bind = create_bind_group(
			device,
			"mc_mesh_bind",
			&self.mesh_layout,
			&[
				&sampling_buf,
				&cube_index,
				&tri_offset,
				&out_pos,
				&out_normals,
				&out_uvs,
				&cfg_buf,
				&bounds_buf,
				&seed_buf,
			],
		);

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
