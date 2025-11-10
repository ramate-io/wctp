use crate::sdf::Sdf;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

/// Trait for elevation modulations that modify terrain height in 2.5D
/// Returns the height offset at a given (x, z) position (Y is ignored)
pub trait ElevationModulation: Send + Sync {
	fn height_offset(&self, x: f32, z: f32) -> f32;
}

/// SDF representation of Perlin noise-based terrain
/// Converts the heightfield `y = height(x, z)` into an SDF: `f(p) = p.y - height(p.x, p.z)`
pub struct PerlinTerrainSdf {
	perlin: Perlin,
	config: TerrainConfig,
	elevation_modulations: Vec<Box<dyn ElevationModulation>>,
}

impl PerlinTerrainSdf {
	pub fn new(seed: u32, config: TerrainConfig) -> Self {
		Self { perlin: Perlin::new(seed), config, elevation_modulations: Vec::new() }
	}

	pub fn add_elevation_modulation(&mut self, modulation: Box<dyn ElevationModulation>) {
		self.elevation_modulations.push(modulation);
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

		let exponent = 1.1; // >1 exaggerates contrast, <1 flattens
		let sign = height.signum();
		let height = sign * height.abs().powf(exponent);
		let height = height * self.config.height_scale;

		height
	}
}

impl Sdf for PerlinTerrainSdf {
	fn distance(&self, p: Vec3) -> f32 {
		// Compute surface height from noise
		let mut terrain_height = self.height_at(p.x, p.z);

		// Apply elevation modulations (2.5D height offsets)
		for modulation in &self.elevation_modulations {
			terrain_height += modulation.height_offset(p.x, p.z);
		}

		// Define bedrock level (bottom of world)
		let bedrock_level = -self.config.height_scale * 4.0;

		// Distance to surface
		let d_surface = p.y - terrain_height;

		// Distance to bedrock (negative below bedrock)
		let d_bedrock = bedrock_level - p.y;

		// Take the maximum (intersection of half-spaces)
		// This keeps the interior solid between surface and bedrock.
		d_surface.max(d_bedrock)
	}
}
