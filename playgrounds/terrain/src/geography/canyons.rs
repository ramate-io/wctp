use crate::geography::GeographicFeature;
use crate::terrain::TerrainConfig;
use noise::{NoiseFn, Perlin};

/// Canyon feature that carves canyons into the terrain
pub struct CanyonFeature {
	perlin: Perlin,
	/// Width of canyon paths (in world units)
	pub width: f32,
	/// Depth of canyons (in world units)
	pub depth: f32,
	/// Frequency of canyon paths (lower = more canyons)
	pub frequency: f32,
	/// How sharply the canyon walls drop off
	pub wall_sharpness: f32,
}

impl CanyonFeature {
	pub fn new(seed: u32, seed_offset: u32) -> Self {
		Self {
			perlin: Perlin::new(seed.wrapping_add(seed_offset)),
			width: 50.0,
			depth: 10.0,
			frequency: 0.01,
			wall_sharpness: 2.0,
		}
	}

	/// Calculate distance from nearest canyon path
	fn distance_to_canyon(&self, world_x: f32, world_z: f32) -> f32 {
		// Use Perlin noise to create meandering canyon paths
		// Sample noise at different scales to create branching canyons
		let noise1 = self
			.perlin
			.get([world_x as f64 * self.frequency as f64, world_z as f64 * self.frequency as f64])
			as f32;
		let noise2 = self.perlin.get([
			world_x as f64 * self.frequency as f64 * 2.0,
			world_z as f64 * self.frequency as f64 * 2.0,
		]) as f32;

		// Combine noise to create canyon network
		let canyon_path = (noise1 + noise2 * 0.5) / 1.5;

		// Distance from canyon center (0 = center, 1 = far away)
		// Use absolute value to create symmetric canyons
		canyon_path.abs()
	}
}

impl GeographicFeature for CanyonFeature {
	fn modify_height(
		&self,
		world_x: f32,
		world_z: f32,
		_base_height: f32,
		_config: &TerrainConfig,
	) -> f32 {
		let distance = self.distance_to_canyon(world_x, world_z);

		// Create canyon shape: deeper in center, shallower at edges
		// Use smoothstep-like function for canyon profile
		let normalized_distance = (distance * 2.0 / self.width).min(1.0);

		// Canyon depth profile: 1.0 at center, 0.0 at edges
		// Use power function for sharper walls
		let depth_factor = (1.0 - normalized_distance).powf(self.wall_sharpness);

		// Only carve if we're within canyon width
		if normalized_distance < 1.0 {
			-depth_factor * self.depth
		} else {
			0.0
		}
	}
}
