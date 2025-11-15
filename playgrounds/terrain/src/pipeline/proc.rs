use crate::terrain::TerrainConfig;
use bevy::{
	prelude::*,
	render::{
		render_resource::*,
		renderer::{RenderDevice, RenderQueue},
	},
};

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Bounds {
	pub enabled: u32,
	pub min: Vec2,
	pub max: Vec2,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Sampling3D {
	pub chunk_origin: Vec3,
	pub chunk_size: Vec3,
	pub resolution: UVec3,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct TerrainConfigGpu {
	pub seed: u32,
	pub base_resolution: u32,
	pub height_scale: f32,
	pub use_volumetric: u32, // bool as u32
}

impl From<&TerrainConfig> for TerrainConfigGpu {
	fn from(config: &TerrainConfig) -> Self {
		Self {
			seed: config.seed,
			base_resolution: config.base_resolution as u32,
			height_scale: config.height_scale,
			use_volumetric: config.use_volumetric as u32,
		}
	}
}

pub struct GpuMarchingCubesPipeline {
	pub sampling: Sampling3D,
	pub terrain_config: TerrainConfig,
	pub bounds: Bounds,
	pub seed: i32,

	// GPU buffers
	cube_index: Buffer,
	tri_counts: Buffer,
	tri_offset: Buffer,

	block_sums: Buffer,
	block_prefix: Buffer,

	out_positions: Buffer,
	out_normals: Buffer,
	out_uvs: Buffer,

	// Pipelines
	classify_pipeline: ComputePipeline,
	prefix_local_pipeline: ComputePipeline,
	prefix_block_pipeline: ComputePipeline,
	prefix_add_pipeline: ComputePipeline,
	mesh_pipeline: ComputePipeline,

	// Bind groups
	classify_bind: BindGroup,
	prefix_local_bind: BindGroup,
	prefix_block_bind: BindGroup,
	prefix_add_bind: BindGroup,
	mesh_bind: BindGroup,

	dispatch_count: UVec3,
	voxel_count: u32,
	block_count: u32,

	total_vertices: u32,
}

impl GpuMarchingCubesPipeline {
	pub fn new(
		device: &RenderDevice,
		pipeline_cache: &mut PipelineCache,
		shaders: &mut Assets<Shader>,
		sampling: Sampling3D,
		terrain_config: TerrainConfig,
		bounds: Bounds,
		seed: i32,
	) -> Self {
		let voxel_count = sampling.resolution.x * sampling.resolution.y * sampling.resolution.z;

		let block_size = 256u32;
		let block_count = (voxel_count + block_size - 1) / block_size;

		// -----------------------------------------
		// Allocate all buffers
		// -----------------------------------------

		let cube_index = device.create_buffer(&BufferDescriptor {
			label: Some("cube index buffer"),
			size: (voxel_count as u64) * 4,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let tri_counts = device.create_buffer(&BufferDescriptor {
			label: Some("tri counts buffer"),
			size: (voxel_count as u64) * 4,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let tri_offset = device.create_buffer(&BufferDescriptor {
			label: Some("tri offset buffer (prefix sum)"),
			size: (voxel_count as u64) * 4,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let block_sums = device.create_buffer(&BufferDescriptor {
			label: Some("block sums buffer"),
			size: (block_count as u64) * 4,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let block_prefix = device.create_buffer(&BufferDescriptor {
			label: Some("block prefix buffer"),
			size: (block_count as u64) * 4,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		// Output buffers (we don't know sizes yet — allocate maximum)
		// 5 triangles per voxel → 15 vertices max
		let max_vertices = voxel_count * 15;

		let out_positions = device.create_buffer(&BufferDescriptor {
			label: Some("out positions"),
			size: (max_vertices as u64) * 12,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
			mapped_at_creation: false,
		});

		let out_normals = device.create_buffer(&BufferDescriptor {
			label: Some("out normals"),
			size: (max_vertices as u64) * 12,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
			mapped_at_creation: false,
		});

		let out_uvs = device.create_buffer(&BufferDescriptor {
			label: Some("out uvs"),
			size: (max_vertices as u64) * 8,
			usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
			mapped_at_creation: false,
		});

		// Dispatch size for 8×8×8 compute
		let dispatch_count = UVec3::new(
			(sampling.resolution.x + 7) / 8,
			(sampling.resolution.y + 7) / 8,
			(sampling.resolution.z + 7) / 8,
		);

		// ----------------------------------------------------------
		// Load and build all compute pipelines
		// ----------------------------------------------------------
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

		let classify_pipeline = create_compute_pipeline(pipeline_cache, &classify_shader, "main");
		let prefix_local_pipeline =
			create_compute_pipeline(pipeline_cache, &prefix_local_shader, "main");
		let prefix_block_pipeline =
			create_compute_pipeline(pipeline_cache, &prefix_block_shader, "main");
		let prefix_add_pipeline =
			create_compute_pipeline(pipeline_cache, &prefix_add_shader, "main");
		let mesh_pipeline = create_compute_pipeline(pipeline_cache, &mesh_shader, "compute_mesh");

		// ----------------------------------------------------------
		// Create bind groups
		// (Helper function shown below)
		// ----------------------------------------------------------

		// Create uniform buffers for structs
		let sampling_buffer = create_uniform_buffer(device, &sampling);
		let terrain_config_gpu = TerrainConfigGpu::from(&terrain_config);
		let terrain_config_buffer = create_uniform_buffer(device, &terrain_config_gpu);
		let bounds_buffer = create_uniform_buffer(device, &bounds);
		let seed_buffer = create_uniform_buffer(device, &seed);

		let classify_bind = create_bind_group(
			device,
			&[
				(&sampling_buffer, BufferBindingType::Uniform),
				(&terrain_config_buffer, BufferBindingType::Uniform),
				(&bounds_buffer, BufferBindingType::Uniform),
				(&seed_buffer, BufferBindingType::Uniform),
				(&cube_index, BufferBindingType::Storage { read_only: false }),
				(&tri_counts, BufferBindingType::Storage { read_only: false }),
			],
		);

		let prefix_local_bind = create_bind_group(
			device,
			&[
				(&tri_counts, BufferBindingType::Storage { read_only: false }),
				(&tri_offset, BufferBindingType::Storage { read_only: false }),
				(&block_sums, BufferBindingType::Storage { read_only: false }),
			],
		);

		let prefix_block_bind = create_bind_group(
			device,
			&[
				(&block_sums, BufferBindingType::Storage { read_only: false }),
				(&block_prefix, BufferBindingType::Storage { read_only: false }),
			],
		);

		let prefix_add_bind = create_bind_group(
			device,
			&[
				(&tri_offset, BufferBindingType::Storage { read_only: false }),
				(&block_prefix, BufferBindingType::Storage { read_only: false }),
			],
		);

		let mesh_bind = create_bind_group(
			device,
			&[
				(&sampling_buffer, BufferBindingType::Uniform),
				(&cube_index, BufferBindingType::Storage { read_only: true }),
				(&tri_offset, BufferBindingType::Storage { read_only: true }),
				(&out_positions, BufferBindingType::Storage { read_only: false }),
				(&out_normals, BufferBindingType::Storage { read_only: false }),
				(&out_uvs, BufferBindingType::Storage { read_only: false }),
				(&terrain_config_buffer, BufferBindingType::Uniform),
				(&bounds_buffer, BufferBindingType::Uniform),
				(&seed_buffer, BufferBindingType::Uniform),
			],
		);

		// ----------------------------------------------------------
		// Construct pipeline struct
		// ----------------------------------------------------------

		Self {
			sampling,
			terrain_config,
			bounds,
			seed,

			cube_index,
			tri_counts,
			tri_offset,

			block_sums,
			block_prefix,

			out_positions,
			out_normals,
			out_uvs,

			classify_pipeline,
			prefix_local_pipeline,
			prefix_block_pipeline,
			prefix_add_pipeline,
			mesh_pipeline,

			classify_bind,
			prefix_local_bind,
			prefix_block_bind,
			prefix_add_bind,
			mesh_bind,

			dispatch_count,
			voxel_count,
			block_count,

			total_vertices: 0,
		}
	}

	pub fn compute(&mut self, device: &RenderDevice, queue: &RenderQueue) {
		let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
			label: Some("marching cubes encoder"),
		});

		// PASS 1: classify
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.classify_pipeline);
			pass.set_bind_group(0, &self.classify_bind, &[]);
			pass.dispatch_workgroups(
				self.dispatch_count.x,
				self.dispatch_count.y,
				self.dispatch_count.z,
			);
		}

		// PASS 2: prefix local
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.prefix_local_pipeline);
			pass.set_bind_group(0, &self.prefix_local_bind, &[]);
			pass.dispatch_workgroups(self.block_count, 1, 1);
		}

		// PASS 3: prefix block scan
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.prefix_block_pipeline);
			pass.set_bind_group(0, &self.prefix_block_bind, &[]);
			pass.dispatch_workgroups(1, 1, 1);
		}

		// PASS 4: prefix add offsets
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.prefix_add_pipeline);
			pass.set_bind_group(0, &self.prefix_add_bind, &[]);
			pass.dispatch_workgroups(self.block_count, 1, 1);
		}

		// PASS 5: compute mesh
		{
			let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
			pass.set_pipeline(&self.mesh_pipeline);
			pass.set_bind_group(0, &self.mesh_bind, &[]);
			pass.dispatch_workgroups(
				self.dispatch_count.x,
				self.dispatch_count.y,
				self.dispatch_count.z,
			);
		}

		queue.submit(std::iter::once(encoder.finish()));
	}
}

// Helper function to create a compute pipeline using PipelineCache
fn create_compute_pipeline(
	pipeline_cache: &mut PipelineCache,
	shader: &Handle<Shader>,
	entry_point: &'static str,
) -> ComputePipeline {
	use std::borrow::Cow;

	// Create the pipeline descriptor
	let pipeline_descriptor = ComputePipelineDescriptor {
		label: Some(Cow::Owned(format!("compute_pipeline_{}", entry_point))),
		layout: vec![],
		shader: shader.clone(),
		shader_defs: vec![],
		entry_point: Cow::Borrowed(entry_point),
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	};

	// Queue the pipeline for compilation
	let pipeline_id = pipeline_cache.queue_compute_pipeline(pipeline_descriptor);

	// Process the pipeline cache to compile the pipeline
	// In a real application, this would happen asynchronously over multiple frames
	// For initialization, we'll process until it's ready
	loop {
		pipeline_cache.process_queue();
		if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) {
			return pipeline.clone();
		}
		// Small yield to prevent tight loop (though this is blocking initialization)
		std::thread::yield_now();
	}
}

// Helper function to create a uniform buffer from a struct
fn create_uniform_buffer<T: bytemuck::Pod>(device: &RenderDevice, data: &T) -> Buffer {
	use bytemuck::bytes_of;
	let bytes = bytes_of(data);
	device.create_buffer_with_data(&BufferInitDescriptor {
		label: Some("uniform_buffer"),
		contents: bytes,
		usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
	})
}

// Helper function to create a bind group
fn create_bind_group(
	device: &RenderDevice,
	bindings: &[(&Buffer, BufferBindingType)],
) -> BindGroup {
	let mut entries = Vec::new();
	let mut layout_entries = Vec::new();

	for (i, (buffer, binding_type)) in bindings.iter().enumerate() {
		let binding = i as u32;
		entries.push(BindGroupEntry {
			binding,
			resource: BindingResource::Buffer(BufferBinding {
				buffer: buffer.clone(),
				offset: 0,
				size: None,
			}),
		});

		layout_entries.push(BindGroupLayoutEntry {
			binding,
			visibility: ShaderStages::COMPUTE,
			ty: match binding_type {
				BufferBindingType::Uniform => BindingType::Buffer {
					ty: BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				&BufferBindingType::Storage { read_only } => BindingType::Buffer {
					ty: BufferBindingType::Storage { read_only },
					has_dynamic_offset: false,
					min_binding_size: None,
				},
			},
			count: None,
		});
	}

	let layout =
		device.create_bind_group_layout(Some("compute_bind_group_layout"), &layout_entries);

	device.create_bind_group(Some("compute_bind_group"), &layout, &entries)
}
