use crate::chunk::{get_chunks_to_load, ChunkConfig, ChunkCoord, LoadedChunks, TerrainChunk};
use crate::cpu::spawn_chunk;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;

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
	// feature_registry: Option<Res<crate::geography::FeatureRegistry>>,
) {
	let Ok(camera_transform) = camera_query.single() else {
		return;
	};

	let camera_pos = camera_transform.translation;

	// Determine which chunks should be loaded
	let chunks_to_load = get_chunks_to_load(camera_pos, &chunk_config);
	let chunks_to_load_set: std::collections::HashSet<ChunkCoord> =
		chunks_to_load.iter().map(|info| info.wrapped).collect();

	let (center_wrapped, _center_unwrapped) = ChunkCoord::from_world_pos(
		camera_pos,
		chunk_config.chunk_size,
		chunk_config.world_size_chunks,
	);

	// Check existing chunks for resolution updates or unloading
	let mut chunks_to_unload = Vec::new();
	let mut chunks_to_regenerate = Vec::new();

	for (entity, chunk) in chunk_query.iter() {
		// Wrap chunk coordinate for comparison
		let wrapped_chunk_coord = if chunk_config.world_size_chunks > 0 {
			chunk.coord.wrap(chunk_config.world_size_chunks)
		} else {
			chunk.coord
		};

		if !chunks_to_load_set.contains(&wrapped_chunk_coord) {
			// Chunk is out of range, unload it
			chunks_to_unload.push((entity, chunk.coord));
		} else {
			// Chunk is still in range, check if resolution needs updating
			let distance = if chunk_config.world_size_chunks > 0 {
				center_wrapped
					.manhattan_distance(&wrapped_chunk_coord, chunk_config.world_size_chunks)
			} else {
				center_wrapped.manhattan_distance(&wrapped_chunk_coord, i32::MAX)
			};
			let required_resolution = terrain_config.resolution_for_distance(distance);

			if chunk.resolution != required_resolution {
				// Resolution changed, need to regenerate
				chunks_to_regenerate.push((entity, wrapped_chunk_coord, required_resolution));
			}
		}
	}

	// Unload chunks that are too far away
	for (entity, coord) in chunks_to_unload {
		commands.entity(entity).despawn();
		loaded_chunks.mark_unloaded(&coord);
		log::debug!("Unloaded chunk at ({}, {})", coord.x, coord.z);
	}

	// Regenerate chunks that need resolution updates
	for (entity, coord, new_resolution) in chunks_to_regenerate {
		commands.entity(entity).despawn();
		loaded_chunks.mark_unloaded(&coord);

		// Respawn at new resolution
		// Find the unwrapped coordinate for this wrapped coord
		let unwrapped_coord = chunks_to_load
			.iter()
			.find(|info| info.wrapped == coord)
			.map(|info| info.unwrapped)
			.unwrap_or(coord);

		let distance = if chunk_config.world_size_chunks > 0 {
			center_wrapped.manhattan_distance(&coord, chunk_config.world_size_chunks)
		} else {
			center_wrapped.manhattan_distance(&coord, i32::MAX)
		};
		spawn_chunk(
			&mut commands,
			&mut meshes,
			&mut materials,
			coord,           // Store wrapped coord for indexing
			unwrapped_coord, // Use unwrapped for world position
			chunk_config.chunk_size,
			chunk_config.world_size_chunks,
			new_resolution,
			&terrain_config,
			// feature_registry.as_deref(),
		);
		loaded_chunks.mark_loaded(coord);
		log::debug!(
			"Regenerated chunk at ({}, {}) from distance {} with resolution {}",
			coord.x,
			coord.z,
			distance,
			new_resolution
		);
	}

	// Load new chunks with appropriate resolution based on distance
	for chunk_info in chunks_to_load {
		if !loaded_chunks.is_loaded(&chunk_info.wrapped) {
			// Calculate Manhattan distance from camera chunk (with wrapping)
			let distance = if chunk_config.world_size_chunks > 0 {
				center_wrapped
					.manhattan_distance(&chunk_info.wrapped, chunk_config.world_size_chunks)
			} else {
				center_wrapped.manhattan_distance(&chunk_info.wrapped, i32::MAX)
			};

			// Get resolution for this distance
			let resolution = terrain_config.resolution_for_distance(distance);

			spawn_chunk(
				&mut commands,
				&mut meshes,
				&mut materials,
				chunk_info.wrapped,   // Store wrapped for indexing
				chunk_info.unwrapped, // Use unwrapped for world position
				chunk_config.chunk_size,
				chunk_config.world_size_chunks,
				resolution,
				&terrain_config,
				// feature_registry.as_deref(),
			);
			loaded_chunks.mark_loaded(chunk_info.wrapped);
		}
	}
}
