use crate::cascade::CascadeChunk;
use bevy::prelude::*;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// Wrapper for Vec3 that implements Hash and Eq for use in HashSet
#[derive(Debug, Clone, Copy)]
pub struct Vec3Key(pub Vec3);

impl PartialEq for Vec3Key {
	fn eq(&self, other: &Self) -> bool {
		self.0.x == other.0.x && self.0.y == other.0.y && self.0.z == other.0.z
	}
}

impl Eq for Vec3Key {}

impl Hash for Vec3Key {
	fn hash<H: Hasher>(&self, state: &mut H) {
		// Hash the float values by converting to a fixed-point representation
		// This is approximate but should work for our use case
		self.0.x.to_bits().hash(state);
		self.0.y.to_bits().hash(state);
		self.0.z.to_bits().hash(state);
	}
}

impl From<Vec3> for Vec3Key {
	fn from(v: Vec3) -> Self {
		Vec3Key(v)
	}
}

impl From<Vec3Key> for Vec3 {
	fn from(k: Vec3Key) -> Self {
		k.0
	}
}

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
	pub chunk: CascadeChunk,
}

/// Resource tracking loaded chunks
/// Uses Vec3 origin as the key for tracking loaded chunks
#[derive(Resource, Default)]
pub struct LoadedChunks {
	pub chunks: HashSet<Vec3Key>,
}

impl LoadedChunks {
	pub fn is_loaded(&self, origin: &Vec3) -> bool {
		self.chunks.contains(&Vec3Key(*origin))
	}

	pub fn mark_loaded(&mut self, origin: Vec3) {
		self.chunks.insert(Vec3Key(origin));
	}

	pub fn mark_unloaded(&mut self, origin: &Vec3) {
		self.chunks.remove(&Vec3Key(*origin));
	}
}

/// Configuration for chunk system using cascade
#[derive(Resource)]
pub struct ChunkConfig {
	/// Minimum chunk size (size of center chunk and ring 0)
	pub min_size: f32,
	/// Number of rings in the cascade
	pub number_of_rings: usize,
	/// World size in world units (for wrapping/torus topology). If 0, no wrapping.
	/// Should be a multiple of cascade span for proper alignment.
	pub world_size: f32,
}

impl Default for ChunkConfig {
	fn default() -> Self {
		Self {
			min_size: 10.0,     // 10 km chunks
			number_of_rings: 4, // 2 rings: center + 2 rings = 3^2 = 9x span = 900m total
			world_size: 0.0,    // No wrapping by default
		}
	}
}
