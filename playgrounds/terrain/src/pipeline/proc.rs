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

use buffers::{new_storage, new_uniform, read_u32, read_vec};
use stages::{ClassifyStage, MeshStage, PrefixAddStage, PrefixBlockStage, PrefixLocalStage};
pub use types::{Bounds, GpuMeshData, Sampling3D, TerrainConfigGpu};

// Re-export main types for convenience

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

	classify_stage: ClassifyStage,
	prefix_local_stage: PrefixLocalStage,
	prefix_block_stage: PrefixBlockStage,
	prefix_add_stage: PrefixAddStage,
	mesh_stage: MeshStage,
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

		// Initialize all stages (creates layouts and loads pipelines)
		let classify_stage = ClassifyStage::new(device, pipeline_cache, asset_server, shaders);
		let prefix_local_stage =
			PrefixLocalStage::new(device, pipeline_cache, asset_server, shaders);
		let prefix_block_stage =
			PrefixBlockStage::new(device, pipeline_cache, asset_server, shaders);
		let prefix_add_stage = PrefixAddStage::new(device, pipeline_cache, asset_server, shaders);
		let mesh_stage = MeshStage::new(device, pipeline_cache, asset_server, shaders);

		Self {
			sampling,
			terrain_cfg: TerrainConfigGpu::from(terrain_cfg_src),
			bounds,
			seed,
			dispatch,
			voxel_count,
			block_count,
			classify_stage,
			prefix_local_stage,
			prefix_block_stage,
			prefix_add_stage,
			mesh_stage,
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
		self.classify_stage.execute(
			device,
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
		self.prefix_local_stage.execute(
			device,
			block_count,
			&tri_counts,
			&tri_offset,
			&block_sums,
			&mut encoder,
		);

		// PASS 3: prefix_block
		self.prefix_block_stage
			.execute(device, &block_sums, &block_prefix, &mut encoder);

		// PASS 4: prefix_add
		self.prefix_add_stage.execute(
			device,
			block_count,
			&tri_offset,
			&block_prefix,
			&mut encoder,
		);

		// PASS 5: mesh
		self.mesh_stage.execute(
			device,
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

/// Helper struct for spawning meshes from GPU-computed mesh data
pub struct TerrainMeshSpawner;

impl TerrainMeshSpawner {
	/// Spawn a mesh entity from GPU-computed mesh data
	pub fn spawn_mesh(
		commands: &mut Commands,
		meshes: &mut Assets<Mesh>,
		materials: &mut Assets<StandardMaterial>,
		data: &GpuMeshData,
		origin: Vec3,
	) -> Entity {
		let mut mesh = Mesh::new(
			PrimitiveTopology::TriangleList,
			RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
		);

		mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.clone());
		mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.clone());
		mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs.clone());

		let mesh_handle = meshes.add(mesh);
		let material_handle = materials.add(StandardMaterial::from(Color::WHITE));

		commands
			.spawn((
				Mesh3d(mesh_handle),
				MeshMaterial3d::<StandardMaterial>(material_handle),
				Transform::from_translation(origin),
			))
			.id()
	}

	/// Convert GPU mesh data to a Bevy Mesh (without spawning)
	pub fn mesh_from_gpu_data(data: &GpuMeshData) -> Mesh {
		let mut mesh = Mesh::new(
			PrimitiveTopology::TriangleList,
			RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
		);

		mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.clone());
		mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.clone());
		mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs.clone());

		mesh
	}
}

/// Legacy function for backward compatibility
#[deprecated(note = "Use TerrainMeshSpawner::spawn_mesh instead")]
pub fn spawn_mesh_from_gpu(
	commands: &mut Commands,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<StandardMaterial>,
	data: &GpuMeshData,
	origin: Vec3,
) {
	TerrainMeshSpawner::spawn_mesh(commands, meshes, materials, data, origin);
}
