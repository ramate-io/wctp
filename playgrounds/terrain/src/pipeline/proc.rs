use crate::{terrain::TerrainConfig, Bounds};
use bevy::{
	prelude::*,
	render::{
		render_resource::*,
		renderer::{Queue, RenderDevice},
	},
};

#[derive(Debug, Clone)]
pub struct Bounds {
	pub enabled: u32,
	pub min: Vec2,
	pub max: Vec2,
}

pub struct Sampling3D {
	pub chunk_origin: Vec3,
	pub chunk_size: Vec3,
	pub resolution: UVec3,
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
		let classify_pipeline =
			create_compute_pipeline(device, "proc/classify_voxels.wgsl", "main");
		let prefix_local_pipeline =
			create_compute_pipeline(device, "proc/prefix_scan_local.wgsl", "main");
		let prefix_block_pipeline =
			create_compute_pipeline(device, "proc/prefix_scan_blocks.wgsl", "main");
		let prefix_add_pipeline =
			create_compute_pipeline(device, "proc/prefix_add_offsets.wgsl", "main");
		let mesh_pipeline =
			create_compute_pipeline(device, "proc/compute_mesh.wgsl", "compute_mesh");

		// ----------------------------------------------------------
		// Create bind groups
		// (Helper function shown below)
		// ----------------------------------------------------------

		let classify_bind = create_bind_group(
			device,
			&classify_pipeline,
			&[
				(&sampling, BufferBindingType::Uniform),
				(&terrain_config, BufferBindingType::Uniform),
				(&bounds, BufferBindingType::Uniform),
				(&seed, BufferBindingType::Uniform),
				(&cube_index, BufferBindingType::Storage),
				(&tri_counts, BufferBindingType::Storage),
			],
		);

		let prefix_local_bind = create_bind_group(
			device,
			&prefix_local_pipeline,
			&[
				(&tri_counts, BufferBindingType::Storage),
				(&tri_offset, BufferBindingType::Storage),
				(&block_sums, BufferBindingType::Storage),
			],
		);

		let prefix_block_bind = create_bind_group(
			device,
			&prefix_block_pipeline,
			&[
				(&block_sums, BufferBindingType::Storage),
				(&block_prefix, BufferBindingType::Storage),
			],
		);

		let prefix_add_bind = create_bind_group(
			device,
			&prefix_add_pipeline,
			&[
				(&tri_offset, BufferBindingType::Storage),
				(&block_prefix, BufferBindingType::Storage),
			],
		);

		let mesh_bind = create_bind_group(
			device,
			&mesh_pipeline,
			&[
				(&sampling, BufferBindingType::Uniform),
				(&cube_index, BufferBindingType::Storage),
				(&tri_offset, BufferBindingType::Storage),
				(&out_positions, BufferBindingType::Storage),
				(&out_normals, BufferBindingType::Storage),
				(&out_uvs, BufferBindingType::Storage),
				(&terrain_config, BufferBindingType::Uniform),
				(&bounds, BufferBindingType::Uniform),
				(&seed, BufferBindingType::Uniform),
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

	pub fn compute(&mut self, device: &RenderDevice, queue: &Queue) {
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
