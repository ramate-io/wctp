use bevy::prelude::*;
use std::fmt::Debug;

pub trait ResolutionMap: Debug + Clone + Copy {
	fn ring_to_resolution(&self, ring: usize) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct Ring {
	pub size: f32,
	pub resolution: usize,
	// the point at the lower left bottom corner of the ring
	pub lower_left_bottom: Vec3,
}

impl Ring {
	pub fn new(size: f32, resolution: usize, lower_left_bottom: Vec3) -> Self {
		Self { size, resolution, lower_left_bottom }
	}

	pub fn ring_chunks(&self) -> Result<RingChunks, String> {
		let mut chunks = Vec::new();
		for x in 0..3 {
			for y in 0..3 {
				for z in 0..3 {
					if x == 1 && y == 1 && z == 1 {
						continue;
					}

					chunks.push(CascadeChunk {
						origin: self.lower_left_bottom
							+ Vec3::new(
								x as f32 * self.size,
								y as f32 * self.size,
								z as f32 * self.size,
							),
						size: self.size,
						resolution: self.resolution,
					});
				}
			}
		}
		RingChunks::try_from(chunks)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct RingChunks {
	pub chunks: [CascadeChunk; 26],
}

impl TryFrom<Vec<CascadeChunk>> for RingChunks {
	type Error = String;

	fn try_from(chunks: Vec<CascadeChunk>) -> Result<Self, Self::Error> {
		if chunks.len() != 26 {
			return Err(format!("Expected 26 chunks, got {}", chunks.len()));
		}
		Ok(Self {
			chunks: chunks
				.try_into()
				.map_err(|e| format!("Failed to convert chunks to array: {:?}", e))?,
		})
	}
}

#[derive(Debug, Clone, Copy)]
pub struct CascadeChunk {
	pub origin: Vec3,
	pub size: f32,
	pub resolution: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Cascade<R: ResolutionMap> {
	pub min_size: f32,
	pub number_of_rings: usize,
	pub resolution_map: R,
}

impl<R: ResolutionMap> Cascade<R> {
	pub fn size_for_ring(&self, ring: usize) -> f32 {
		self.min_size * (3 * ring) as f32
	}

	pub fn position_to_origin(&self, position: Vec3) -> Vec3 {
		let x = (position.x / self.min_size).floor() * self.min_size;
		let y = (position.y / self.min_size).floor() * self.min_size;
		let z = (position.z / self.min_size).floor() * self.min_size;
		Vec3::new(x, y, z)
	}

	pub fn center_chunk(&self, position: Vec3) -> CascadeChunk {
		let origin = self.position_to_origin(position);
		CascadeChunk {
			origin,
			size: self.min_size,
			resolution: self.resolution_map.ring_to_resolution(0),
		}
	}

	pub fn chunks(&self, position: Vec3) -> Result<Vec<CascadeChunk>, String> {
		let center_chunk = self.center_chunk(position);
		let mut last_lower_left_bottom = center_chunk.origin;
		let mut chunks = Vec::new();
		chunks.push(center_chunk);
		for ring in 0..self.number_of_rings {
			let size = self.size_for_ring(ring);
			let new_lower_left_bottom = last_lower_left_bottom - Vec3::new(size, size, size);
			let ring_chunks = Ring::new(
				size,
				self.resolution_map.ring_to_resolution(ring),
				new_lower_left_bottom,
			)
			.ring_chunks()?;
			chunks.extend(ring_chunks.chunks);
			last_lower_left_bottom = new_lower_left_bottom;
		}
		Ok(chunks)
	}
}
