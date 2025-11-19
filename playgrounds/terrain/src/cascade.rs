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

fn lex_cmp(a: &Vec3, b: &Vec3) -> std::cmp::Ordering {
	a.x.partial_cmp(&b.x)
		.unwrap_or(std::cmp::Ordering::Equal)
		.then_with(|| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
		.then_with(|| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CascadeChunk {
	pub origin: Vec3,
	pub size: f32,
	pub resolution: usize,
}

impl PartialOrd for CascadeChunk {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// compare the size first, then the resolution, then the origin
		Some(
			self.size
				.partial_cmp(&other.size)
				.unwrap_or(std::cmp::Ordering::Equal)
				.then_with(|| {
					self.resolution
						.partial_cmp(&other.resolution)
						.unwrap_or(std::cmp::Ordering::Equal)
				})
				.then_with(|| lex_cmp(&self.origin, &other.origin)),
		)
	}
}

#[cfg(test)]
impl Eq for CascadeChunk {}

#[cfg(test)]
impl Ord for CascadeChunk {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.partial_cmp(other).unwrap()
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Cascade<R: ResolutionMap> {
	pub min_size: f32,
	pub number_of_rings: usize,
	pub resolution_map: R,
}

impl<R: ResolutionMap> Cascade<R> {
	pub fn size_for_ring(&self, ring: usize) -> f32 {
		self.min_size * 3_u32.pow(ring as u32) as f32
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
		// copmute the center chunk
		let center_chunk = self.center_chunk(position);

		// move to the lower bottom left for the 0th ring
		let mut lower_left_bottom =
			center_chunk.origin - Vec3::new(self.min_size, self.min_size, self.min_size);

		// add the center chunk to the chunks vector
		let mut chunks = Vec::new();
		chunks.push(center_chunk);

		// iterate over the rings and add the chunks to the chunks vector
		for ring in 0..self.number_of_rings {
			// slightly redundant here,
			// as this is the same as the next_size at the end of the previous iteration
			// but it's IMO clearer to have the size here
			let size = self.size_for_ring(ring);

			// create the ring chunks
			let ring_chunks =
				Ring::new(size, self.resolution_map.ring_to_resolution(ring), lower_left_bottom)
					.ring_chunks()?;

			// add the ring chunks to the chunks vector
			chunks.extend(ring_chunks.chunks);

			// move to the new lower bottom left for the next ring
			let next_size = self.size_for_ring(ring + 1);
			lower_left_bottom = lower_left_bottom - Vec3::new(next_size, next_size, next_size);
		}
		Ok(chunks)
	}

	pub fn needs_new_chunks(&self, prev: Vec3, new: Vec3) -> bool {
		self.position_to_origin(prev) != self.position_to_origin(new)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct ConstantResolutionMap {
	pub resolution: usize,
}

impl ResolutionMap for ConstantResolutionMap {
	fn ring_to_resolution(&self, _ring: usize) -> usize {
		self.resolution
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::BTreeSet;

	fn zeroth_ring(size: f32, resolution: usize) -> Vec<CascadeChunk> {
		// Explicit reference for all 27 chunks of a 3x3x3 cube
		// Organized by z-level: z = -1, z = 0, z = 1
		vec![
			// z = -1 level (9 chunks)
			CascadeChunk {
				origin: Vec3::new(-1.0 * size, -1.0 * size, -1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(0.0 * size, -1.0 * size, -1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(1.0 * size, -1.0 * size, -1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(-1.0 * size, 0.0 * size, -1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(0.0 * size, 0.0 * size, -1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(1.0 * size, 0.0 * size, -1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(-1.0 * size, 1.0 * size, -1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(0.0 * size, 1.0 * size, -1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(1.0 * size, 1.0 * size, -1.0 * size),
				size,
				resolution,
			},
			// z = 0 level (9 chunks)
			CascadeChunk {
				origin: Vec3::new(-1.0 * size, -1.0 * size, 0.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(0.0 * size, -1.0 * size, 0.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(1.0 * size, -1.0 * size, 0.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(-1.0 * size, 0.0 * size, 0.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(0.0 * size, 0.0 * size, 0.0 * size),
				size,
				resolution,
			}, // center
			CascadeChunk {
				origin: Vec3::new(1.0 * size, 0.0 * size, 0.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(-1.0 * size, 1.0 * size, 0.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(0.0 * size, 1.0 * size, 0.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(1.0 * size, 1.0 * size, 0.0 * size),
				size,
				resolution,
			},
			// z = 1 level (9 chunks)
			CascadeChunk {
				origin: Vec3::new(-1.0 * size, -1.0 * size, 1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(0.0 * size, -1.0 * size, 1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(1.0 * size, -1.0 * size, 1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(-1.0 * size, 0.0 * size, 1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(0.0 * size, 0.0 * size, 1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(1.0 * size, 0.0 * size, 1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(-1.0 * size, 1.0 * size, 1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(0.0 * size, 1.0 * size, 1.0 * size),
				size,
				resolution,
			},
			CascadeChunk {
				origin: Vec3::new(1.0 * size, 1.0 * size, 1.0 * size),
				size,
				resolution,
			},
		]
	}

	fn nth_ring(size: f32, resolution: usize) -> Vec<CascadeChunk> {
		// Get the zeroth ring (all 27 chunks)
		let all_chunks = zeroth_ring(size, resolution);

		// Convert to BTreeSet to easily remove the middle chunk
		let mut chunks_set: BTreeSet<CascadeChunk> = all_chunks.into_iter().collect();

		// Remove the middle chunk (center at origin 0,0,0)
		let center_chunk = CascadeChunk { origin: Vec3::new(0.0, 0.0, 0.0), size, resolution };
		chunks_set.remove(&center_chunk);

		// Convert back to Vec
		chunks_set.into_iter().collect()
	}

	#[test]
	fn test_cascade_ones() -> Result<(), String> {
		let cascade = Cascade {
			min_size: 1.0,
			number_of_rings: 1,
			resolution_map: ConstantResolutionMap { resolution: 1 },
		};
		let chunks = cascade.chunks(Vec3::new(0.0, 0.0, 0.0))?;

		// the zero chunk and the 26 chunks in the first ring
		assert_eq!(chunks.len(), 27);

		// put the chunks in a BTreeSet to sort them
		let mut chunks_sorted = BTreeSet::new();
		for chunk in chunks {
			chunks_sorted.insert(chunk);
		}

		// make sure there are 27 unique chunks
		assert_eq!(chunks_sorted.len(), 27);

		// now form a reference vector for the 3x3x3 cube
		let reference_chunks_vec = zeroth_ring(1.0, 1);
		assert_eq!(reference_chunks_vec.len(), 27);

		// Convert reference to BTreeSet for comparison
		let mut reference_sorted = BTreeSet::new();
		for chunk in reference_chunks_vec {
			reference_sorted.insert(chunk);
		}

		// Assert that we build out the 3x3x3 cube correctly
		assert_eq!(chunks_sorted, reference_sorted);

		Ok(())
	}
}
