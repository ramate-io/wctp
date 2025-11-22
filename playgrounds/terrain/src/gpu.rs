use crate::cascade::CascadeChunk;
use crate::chunk::TerrainChunk;
use crate::pipeline::proc::pipelines_resource::MarchingCubesPipelines;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};

/// GPU-based terrain mesh generator
pub struct GpuMeshGenerator;

impl GpuMeshGenerator {
	/// Generate a terrain mesh for a specific chunk using GPU-based Marching Cubes
	/// NOTE: GPU implementation is still being sorted, this is a stub
	pub fn generate_chunk_mesh(
		_cascade_chunk: &CascadeChunk,
		_config: &TerrainConfig,
		_device: &RenderDevice,
		_queue: &RenderQueue,
		_pipelines: &MarchingCubesPipelines,
	) -> Mesh {
		// TODO: Implement GPU mesh generation with cascade chunks
		// For now, return an empty mesh
		Mesh::new(
			bevy::mesh::PrimitiveTopology::TriangleList,
			bevy::asset::RenderAssetUsages::RENDER_WORLD,
		)
	}

	/// Spawn a terrain chunk entity using GPU mesh generation
	/// NOTE: GPU implementation is still being sorted, this is a stub
	pub fn spawn_chunk(
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
		cascade_chunk: CascadeChunk,
		config: &TerrainConfig,
		device: &RenderDevice,
		queue: &RenderQueue,
		pipelines: &MarchingCubesPipelines,
	) -> Entity {
		// Use cascade chunk for mesh generation
		let mesh = Self::generate_chunk_mesh(&cascade_chunk, config, device, queue, pipelines);
		let mesh_handle = meshes.add(mesh);

		let base_color = Color::hsla(46.0, 0.22, 0.62, 1.0); // brown

		let material_handle = materials.add(StandardMaterial {
			base_color,
			metallic: 0.0,
			perceptual_roughness: 0.7,
			..default()
		});

		// Use cascade chunk origin for world position
		let world_pos = cascade_chunk.origin;

		let entity = commands
			.spawn((
				TerrainChunk { chunk: cascade_chunk },
				Mesh3d(mesh_handle.clone()),
				MeshMaterial3d::<StandardMaterial>(material_handle.clone()),
				Transform::from_translation(world_pos),
			))
			.id();

		log::debug!(
			"Spawned chunk (GPU stub) at origin {:?} with size {} and resolution {}",
			cascade_chunk.origin,
			cascade_chunk.size,
			cascade_chunk.res_2
		);

		entity
	}
}
