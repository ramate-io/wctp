// =================================================================================================
// GPU MARCHING CUBES — TYPE SAFE PIPELINE
// =================================================================================================
// Bevy 0.17 — fully self-contained and overwrite-safe
// compute() returns immutable CPU-side mesh data
//
// This module implements a GPU-based Marching Cubes algorithm using compute shaders.
// The pipeline is split into 5 passes:
//   1. classify: Classify voxels and compute cube indices
//   2. prefix_local: Local prefix scan of triangle counts
//   3. prefix_block: Block-level prefix scan
//   4. prefix_add: Add block prefixes to local offsets
//   5. mesh: Generate mesh vertices, normals, and UVs
//
// WGSL binding contract (must match these in your shaders):
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
// compute_mesh.wgsl:
//   @group(0) @binding(0) var<uniform> sampling : Sampling3D;
//   @group(0) @binding(1) var<storage, read> cube_index : array<u32>;
//   @group(0) @binding(2) var<storage, read> tri_offset : array<u32>;
//   @group(0) @binding(3) var<storage, read_write> out_positions : array<vec3<f32>>;
//   @group(0) @binding(4) var<storage, read_write> out_normals   : array<vec3<f32>>;
//   @group(0) @binding(5) var<storage, read_write> out_uvs       : array<vec2<f32>>;
//   @group(0) @binding(6) var<uniform> terrain_config : TerrainConfig;
//   @group(0) @binding(7) var<uniform> bounds : Bounds;
//   @group(0) @binding(8) var<uniform> seed : i32;

mod bind_groups;
mod buffers;
mod pipelines;
mod stages;
mod types;

use bind_groups::{create_storage_layout_entry, create_uniform_layout_entry};
use buffers::{new_storage, new_uniform, read_u32, read_vec};
use pipelines::load_pipeline;
use stages::{
	stage_classify, stage_mesh, stage_prefix_add, stage_prefix_block, stage_prefix_local,
};
pub use types::{Bounds, GpuMeshData, Sampling3D, TerrainConfigGpu};

use bevy::{
	prelude::*,
	render::{
		render_asset::RenderAssetUsages,
		render_resource::*,
		renderer::{RenderDevice, RenderQueue},
	},
};

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
		asset_server: &AssetServer,
		shaders: &Assets<Shader>,
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

		// Pipelines - loaded via asset paths to support #import syntax
		// Note: Asset paths are relative to the assets/ directory
		let classify_pipeline = load_pipeline(
			pipeline_cache,
			asset_server,
			shaders,
			"proc/classify_voxels.wgsl",
			"main",
			&classify_layout,
		);
		let prefix_local_pipeline = load_pipeline(
			pipeline_cache,
			asset_server,
			shaders,
			"proc/prefix_scan_local.wgsl",
			"main",
			&prefix_local_layout,
		);
		let prefix_block_pipeline = load_pipeline(
			pipeline_cache,
			asset_server,
			shaders,
			"proc/block_prefix.wgsl",
			"main",
			&prefix_block_layout,
		);
		let prefix_add_pipeline = load_pipeline(
			pipeline_cache,
			asset_server,
			shaders,
			"proc/block_sum.wgsl",
			"main",
			&prefix_add_layout,
		);
		let mesh_pipeline = load_pipeline(
			pipeline_cache,
			asset_server,
			shaders,
			"proc/compute_mesh.wgsl",
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

		// ---------------- Run compute passes ----------------
		let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());

		// PASS 1: classify
		stage_classify(
			device,
			&self.classify_layout,
			&self.classify_pipeline,
			self.dispatch,
			&sampling_buf,
			&cfg_buf,
			&bounds_buf,
			&seed_buf,
			&cube_index,
			&tri_counts,
			&mut encoder,
		);

		// PASS 2: prefix_local
		stage_prefix_local(
			device,
			&self.prefix_local_layout,
			&self.prefix_local_pipeline,
			block_count,
			&tri_counts,
			&tri_offset,
			&block_sums,
			&mut encoder,
		);

		// PASS 3: prefix_block
		stage_prefix_block(
			device,
			&self.prefix_block_layout,
			&self.prefix_block_pipeline,
			&block_sums,
			&block_prefix,
			&mut encoder,
		);

		// PASS 4: prefix_add
		stage_prefix_add(
			device,
			&self.prefix_add_layout,
			&self.prefix_add_pipeline,
			block_count,
			&tri_offset,
			&block_prefix,
			&mut encoder,
		);

		// PASS 5: mesh
		stage_mesh(
			device,
			&self.mesh_layout,
			&self.mesh_pipeline,
			self.dispatch,
			&sampling_buf,
			&cube_index,
			&tri_offset,
			&out_pos,
			&out_normals,
			&out_uvs,
			&cfg_buf,
			&bounds_buf,
			&seed_buf,
			&mut encoder,
		);

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
