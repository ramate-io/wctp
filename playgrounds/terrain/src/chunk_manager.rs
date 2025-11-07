use bevy::prelude::*;
use noise::Perlin;
use crate::chunk::{ChunkCoord, ChunkConfig, LoadedChunks, TerrainChunk, get_chunks_to_load};
use crate::terrain::{TerrainConfig, spawn_chunk};

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
) {
	let Ok(camera_transform) = camera_query.single() else {
		return;
	};

	let camera_pos = camera_transform.translation;

	// Determine which chunks should be loaded
	let chunks_to_load = get_chunks_to_load(camera_pos, &chunk_config);
	let chunks_to_load_set: std::collections::HashSet<ChunkCoord> =
		chunks_to_load.iter().copied().collect();

	// Create noise generator (reused for all chunks)
	let perlin = Perlin::new(terrain_config.seed);

	// Unload chunks that are too far away
	let mut chunks_to_unload = Vec::new();
	for (entity, chunk) in chunk_query.iter() {
		if !chunks_to_load_set.contains(&chunk.coord) {
			chunks_to_unload.push((entity, chunk.coord));
		}
	}

	for (entity, coord) in chunks_to_unload {
		commands.entity(entity).despawn();
		loaded_chunks.mark_unloaded(&coord);
		log::debug!("Unloaded chunk at ({}, {})", coord.x, coord.z);
	}

	// Load new chunks
	for coord in chunks_to_load {
		if !loaded_chunks.is_loaded(&coord) {
			spawn_chunk(
				&mut commands,
				&mut meshes,
				&mut materials,
				coord,
				chunk_config.chunk_size,
				&terrain_config,
				&perlin,
			);
			loaded_chunks.mark_loaded(coord);
		}
	}
}

