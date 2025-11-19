use crate::cascade::{Cascade, ConstantResolutionMap};
use crate::chunk::{ChunkConfig, LoadedChunks, TerrainChunk, Vec3Key};
use crate::mesh_generator::{MeshGenerationMode, MeshGenerator};
use crate::pipeline::proc::pipelines_resource::MarchingCubesPipelines;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use std::collections::HashSet;

/// Helper function to wrap a Vec3 coordinate within world bounds
/// If world_size is 0, returns the coordinate unchanged (no wrapping)
fn wrap_coordinate(pos: Vec3, world_size: f32) -> Vec3 {
	if world_size <= 0.0 {
		return pos;
	}
	Vec3::new(
		((pos.x % world_size) + world_size) % world_size,
		((pos.y % world_size) + world_size) % world_size,
		((pos.z % world_size) + world_size) % world_size,
	)
}

/// System that manages chunk loading and unloading based on camera position
pub fn manage_chunks(
	mut commands: Commands,
	camera_query: Query<&Transform, With<Camera3d>>,
	chunk_query: Query<(Entity, &TerrainChunk)>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	chunk_config: Res<ChunkConfig>,
	terrain_config: Res<TerrainConfig>,
	mut loaded_chunks: ResMut<LoadedChunks>,
	mesh_mode: Res<MeshGenerationMode>,
	// GPU resources (required for GPU mode, optional for CPU mode)
	render_device: Option<Res<RenderDevice>>,
	render_queue: Option<Res<RenderQueue>>,
	pipelines: Option<Res<MarchingCubesPipelines>>,
	// feature_registry: Option<Res<crate::geography::FeatureRegistry>>,
) {
	// Early return if GPU mode is requested but resources aren't available yet
	if *mesh_mode == MeshGenerationMode::Gpu {
		log::info!(
			"GPU mode requested, checking resources: {:?} {:?} {:?}",
			render_device.is_some(),
			render_queue.is_some(),
			pipelines.is_some(),
		);
		if render_device.is_none() || render_queue.is_none() || pipelines.is_none() {
			warn!("GPU mode requested but resources aren't available");
			return;
		}
	}
	let Ok(camera_transform) = camera_query.single() else {
		return;
	};

	let camera_pos = camera_transform.translation;

	// Create cascade instance
	let cascade = Cascade {
		min_size: chunk_config.min_size,
		number_of_rings: chunk_config.number_of_rings,
		resolution_map: ConstantResolutionMap { resolution: terrain_config.base_resolution },
	};

	// Get chunks from cascade
	let chunks_to_load = match cascade.chunks(camera_pos) {
		Ok(chunks) => chunks,
		Err(e) => {
			log::error!("Failed to get cascade chunks: {}", e);
			return;
		}
	};

	// Create set of chunk origins for quick lookup (with wrapping)
	let chunks_to_load_set: HashSet<Vec3Key> = chunks_to_load
		.iter()
		.map(|chunk| {
			let wrapped_origin = if chunk_config.world_size > 0.0 {
				wrap_coordinate(chunk.origin, chunk_config.world_size)
			} else {
				chunk.origin
			};
			Vec3Key(wrapped_origin)
		})
		.collect();

	// Helper to wrap a chunk origin
	let wrap_chunk_origin = |origin: Vec3| -> Vec3 {
		if chunk_config.world_size > 0.0 {
			wrap_coordinate(origin, chunk_config.world_size)
		} else {
			origin
		}
	};

	// Check existing chunks for unloading
	let mut chunks_to_unload = Vec::new();
	for (entity, chunk) in chunk_query.iter() {
		let wrapped_origin = wrap_chunk_origin(chunk.chunk.origin);
		if !chunks_to_load_set.contains(&Vec3Key(wrapped_origin)) {
			chunks_to_unload.push((entity, chunk.chunk.origin));
		}
	}

	// Unload chunks that are too far away
	for (entity, origin) in chunks_to_unload {
		commands.entity(entity).despawn();
		loaded_chunks.mark_unloaded(&wrap_chunk_origin(origin));
		log::debug!("Unloaded chunk at {:?}", origin);
	}

	// Load new chunks from cascade
	for cascade_chunk in chunks_to_load {
		let wrapped_origin = wrap_chunk_origin(cascade_chunk.origin);
		if !loaded_chunks.is_loaded(&wrapped_origin) {
			MeshGenerator::spawn_chunk(
				*mesh_mode,
				&mut commands,
				&mut meshes,
				&mut materials,
				cascade_chunk,
				&terrain_config,
				render_device.as_deref(),
				render_queue.as_deref(),
				pipelines.as_deref(),
			);
			loaded_chunks.mark_loaded(wrapped_origin);
		}
	}
}
