use crate::sdf::Sdf;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

/// SDF representation of Perlin noise-based terrain
/// Converts the heightfield `y = height(x, z)` into an SDF: `f(p) = p.y - height(p.x, p.z)`
pub struct PerlinTerrainSdf {
	perlin: Perlin,
	config: TerrainConfig,
}

impl PerlinTerrainSdf {
	pub fn new(seed: u32, config: TerrainConfig) -> Self {
		Self { perlin: Perlin::new(seed), config }
	}

	/// Calculate the terrain height at a given (x, z) position
	/// This is the same logic as the original heightfield generation
	fn height_at(&self, world_x: f32, world_z: f32) -> f32 {
		// Generate height using multiple octaves of noise
		let mut height = 0.0;
		let mut amplitude = 1.0;
		let mut frequency = 0.05;
		// let max_value = 0.0;

		for _ in 0..4 {
			let sample =
				self.perlin.get([world_x as f64 * frequency, world_z as f64 * frequency]) as f32;
			height += sample * amplitude;
			// max_value += amplitude;
			amplitude *= 0.5;
			frequency *= 2.0;
		}

		let exponent = 1.2; // >1 exaggerates contrast, <1 flattens
		let sign = height.signum();
		let height = sign * height.abs().powf(exponent);
		let height = height * self.config.height_scale;

		// Apply geographic features (canyons, etc.)
		/*if let Some(registry) = &self.feature_registry {
			height = registry.apply_features(world_x, world_z, height, &self.config);
		}*/

		height
	}
}

impl Sdf for PerlinTerrainSdf {
	fn distance(&self, p: Vec3) -> f32 {
		// Compute surface height from noise
		let terrain_height = self.height_at(p.x, p.z);

		// Define bedrock level (bottom of world)
		let bedrock_level = -self.config.height_scale * 2.0;

		// Distance to surface
		let d_surface = p.y - terrain_height;

		// Distance to bedrock (negative below bedrock)
		let d_bedrock = bedrock_level - p.y;

		// Take the maximum (intersection of half-spaces)
		// This keeps the interior solid between surface and bedrock.
		d_surface.max(d_bedrock)
	}
}
