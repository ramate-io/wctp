use bevy::prelude::*;
use std::collections::HashSet;

/// Chunk coordinate in the world grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
	pub x: i32,
	pub z: i32,
}

impl ChunkCoord {
	pub fn new(x: i32, z: i32) -> Self {
		Self { x, z }
	}

	/// Convert world position to chunk coordinate
	pub fn from_world_pos(world_pos: Vec3, chunk_size: f32) -> Self {
		Self {
			x: (world_pos.x / chunk_size).floor() as i32,
			z: (world_pos.z / chunk_size).floor() as i32,
		}
	}

	/// Get world position of chunk center
	pub fn to_world_pos(&self, chunk_size: f32) -> Vec3 {
		Vec3::new((self.x as f32 + 0.5) * chunk_size, 0.0, (self.z as f32 + 0.5) * chunk_size)
	}

	/// Get world position of chunk origin (corner)
	pub fn to_world_origin(&self, chunk_size: f32) -> Vec3 {
		Vec3::new(self.x as f32 * chunk_size, 0.0, self.z as f32 * chunk_size)
	}

	/// Calculate Manhattan distance between chunks
	pub fn manhattan_distance(&self, other: &Self) -> i32 {
		(self.x - other.x).abs() + (self.z - other.z).abs()
	}
}

/// Component marking a terrain chunk entity
#[derive(Component, Debug, Clone, Copy)]
pub struct TerrainChunk {
	pub coord: ChunkCoord,
}

/// Resource tracking loaded chunks
#[derive(Resource, Default)]
pub struct LoadedChunks {
	pub chunks: HashSet<ChunkCoord>,
}

impl LoadedChunks {
	pub fn is_loaded(&self, coord: &ChunkCoord) -> bool {
		self.chunks.contains(coord)
	}

	pub fn mark_loaded(&mut self, coord: ChunkCoord) {
		self.chunks.insert(coord);
	}

	pub fn mark_unloaded(&mut self, coord: &ChunkCoord) {
		self.chunks.remove(coord);
	}
}

/// Configuration for chunk system
#[derive(Resource)]
pub struct ChunkConfig {
	/// Size of each chunk in world units
	pub chunk_size: f32,
	/// Number of chunks to load in each direction from camera (square radius)
	pub load_radius: i32,
	/// Maximum distance to render chunks (in chunk units)
	pub max_render_distance: i32,
}

impl Default for ChunkConfig {
	fn default() -> Self {
		Self { chunk_size: 100.0, load_radius: 3, max_render_distance: 5 }
	}
}

/// Get all chunk coordinates that should be loaded around a position
/// Uses square approximation of concentric circles
pub fn get_chunks_to_load(camera_pos: Vec3, config: &ChunkConfig) -> Vec<ChunkCoord> {
	let center_chunk = ChunkCoord::from_world_pos(camera_pos, config.chunk_size);
	let mut chunks = Vec::new();

	// Square-based loading: load all chunks within radius
	for dx in -config.load_radius..=config.load_radius {
		for dz in -config.load_radius..=config.load_radius {
			let coord = ChunkCoord::new(center_chunk.x + dx, center_chunk.z + dz);

			// Check if within max render distance
			let distance = center_chunk.manhattan_distance(&coord);
			if distance <= config.max_render_distance {
				chunks.push(coord);
			}
		}
	}

	chunks
}
