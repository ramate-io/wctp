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

	/// Wrap chunk coordinates to world bounds (torus topology)
	pub fn wrap(&self, world_size_chunks: i32) -> Self {
		Self {
			x: ((self.x % world_size_chunks) + world_size_chunks) % world_size_chunks,
			z: ((self.z % world_size_chunks) + world_size_chunks) % world_size_chunks,
		}
	}

	/// Convert world position to chunk coordinate (with wrapping)
	/// Returns both the wrapped coordinate and the "display" coordinate for spawning
	pub fn from_world_pos(
		world_pos: Vec3,
		chunk_size: f32,
		world_size_chunks: i32,
	) -> (Self, Self) {
		// Calculate unwrapped chunk coordinate
		let unwrapped = Self {
			x: (world_pos.x / chunk_size).floor() as i32,
			z: (world_pos.z / chunk_size).floor() as i32,
		};

		// Calculate wrapped coordinate for indexing
		let wrapped =
			if world_size_chunks > 0 { unwrapped.wrap(world_size_chunks) } else { unwrapped };

		(wrapped, unwrapped)
	}

	/// Get world position of chunk center (with wrapping)
	pub fn to_world_pos(&self, chunk_size: f32, world_size_chunks: i32) -> Vec3 {
		let wrapped = self.wrap(world_size_chunks);
		let x = (wrapped.x as f32 + 0.5) * chunk_size;
		let z = (wrapped.z as f32 + 0.5) * chunk_size;
		Vec3::new(x, 0.0, z)
	}

	/// Get world position of chunk origin (corner) with wrapping
	/// If use_wrapped_pos is true, uses wrapped coordinates; otherwise uses unwrapped
	pub fn to_world_origin(
		&self,
		chunk_size: f32,
		world_size_chunks: i32,
		use_wrapped_pos: bool,
	) -> Vec3 {
		if use_wrapped_pos && world_size_chunks > 0 {
			let wrapped = self.wrap(world_size_chunks);
			Vec3::new(wrapped.x as f32 * chunk_size, 0.0, wrapped.z as f32 * chunk_size)
		} else {
			Vec3::new(self.x as f32 * chunk_size, 0.0, self.z as f32 * chunk_size)
		}
	}

	/// Get unwrapped world position for noise generation (allows seamless wrapping)
	pub fn to_unwrapped_world_pos(&self, chunk_size: f32) -> Vec3 {
		Vec3::new(self.x as f32 * chunk_size, 0.0, self.z as f32 * chunk_size)
	}

	/// Calculate Manhattan distance between chunks (accounting for wrapping)
	pub fn manhattan_distance(&self, other: &Self, world_size_chunks: i32) -> i32 {
		let wrapped_self = self.wrap(world_size_chunks);
		let wrapped_other = other.wrap(world_size_chunks);

		// Calculate distance in both directions (wrapped and unwrapped)
		let dx = (wrapped_self.x - wrapped_other.x).abs();
		let dz = (wrapped_self.z - wrapped_other.z).abs();

		// Account for wrapping - use the shorter path
		let dx_wrapped = world_size_chunks - dx;
		let dz_wrapped = world_size_chunks - dz;

		let dx_min = dx.min(dx_wrapped);
		let dz_min = dz.min(dz_wrapped);

		dx_min + dz_min
	}
}

/// Component marking a terrain chunk entity
#[derive(Component, Debug, Clone, Copy)]
pub struct TerrainChunk {
	pub coord: ChunkCoord,
	pub resolution: usize,
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
	/// World size in chunks (for wrapping/torus topology). If 0, no wrapping.
	pub world_size_chunks: i32,
}

impl Default for ChunkConfig {
	fn default() -> Self {
		Self {
			chunk_size: 100.0,
			load_radius: 3,
			max_render_distance: 5,
			world_size_chunks: 32, // 32x32 chunks = 3200x3200 world units, wraps around
		}
	}
}

/// Chunk info for loading - contains both wrapped (for indexing) and unwrapped (for display) coordinates
#[derive(Debug, Clone, Copy)]
pub struct ChunkLoadInfo {
	pub wrapped: ChunkCoord,   // For indexing/uniqueness
	pub unwrapped: ChunkCoord, // For world position
}

/// Get all chunk coordinates that should be loaded around a position
/// Uses square approximation of concentric circles
/// Handles wrapping for spherical/torus topology
pub fn get_chunks_to_load(camera_pos: Vec3, config: &ChunkConfig) -> Vec<ChunkLoadInfo> {
	let (center_wrapped, center_unwrapped) =
		ChunkCoord::from_world_pos(camera_pos, config.chunk_size, config.world_size_chunks);
	let mut chunks = Vec::new();

	// Square-based loading: load all chunks within radius
	for dx in -config.load_radius..=config.load_radius {
		for dz in -config.load_radius..=config.load_radius {
			let unwrapped_coord = ChunkCoord::new(center_unwrapped.x + dx, center_unwrapped.z + dz);

			// Wrap coordinates if world is finite
			let wrapped_coord = if config.world_size_chunks > 0 {
				unwrapped_coord.wrap(config.world_size_chunks)
			} else {
				unwrapped_coord
			};

			// Check if within max render distance (using wrapped distance)
			let distance = if config.world_size_chunks > 0 {
				center_wrapped.manhattan_distance(&wrapped_coord, config.world_size_chunks)
			} else {
				center_wrapped.manhattan_distance(&wrapped_coord, i32::MAX)
			};

			if distance <= config.max_render_distance {
				chunks.push(ChunkLoadInfo { wrapped: wrapped_coord, unwrapped: unwrapped_coord });
			}
		}
	}

	chunks
}
