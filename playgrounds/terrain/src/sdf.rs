use crate::geography::FeatureRegistry;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

/// Trait for Signed Distance Fields
/// Returns the signed distance from a point to the surface:
/// - Negative: inside/below the surface
/// - Zero: on the surface
/// - Positive: outside/above the surface
pub trait Sdf: Send + Sync {
	fn distance(&self, p: Vec3) -> f32;
}

/// SDF representation of Perlin noise-based terrain
/// Converts the heightfield `y = height(x, z)` into an SDF: `f(p) = p.y - height(p.x, p.z)`
pub struct PerlinTerrainSdf<'a> {
	perlin: Perlin,
	config: TerrainConfig,
	feature_registry: Option<&'a FeatureRegistry>,
}

impl<'a> PerlinTerrainSdf<'a> {
	pub fn new(
		seed: u32,
		config: TerrainConfig,
		feature_registry: Option<&'a FeatureRegistry>,
	) -> Self {
		Self { perlin: Perlin::new(seed), config, feature_registry }
	}

	/// Calculate the terrain height at a given (x, z) position
	/// This is the same logic as the original heightfield generation
	fn height_at(&self, world_x: f32, world_z: f32) -> f32 {
		// Generate height using multiple octaves of noise
		let mut height = 0.0;
		let mut amplitude = 1.0;
		let mut frequency = 0.05;
		let mut max_value = 0.0;

		for _ in 0..4 {
			let sample =
				self.perlin.get([world_x as f64 * frequency, world_z as f64 * frequency]) as f32;
			height += sample * amplitude;
			max_value += amplitude;
			amplitude *= 0.5;
			frequency *= 2.0;
		}

		height = (height / max_value) * self.config.height_scale;

		// Apply geographic features (canyons, etc.)
		if let Some(registry) = &self.feature_registry {
			height = registry.apply_features(world_x, world_z, height, &self.config);
		}

		height
	}
}

impl<'a> Sdf for PerlinTerrainSdf<'a> {
	fn distance(&self, p: Vec3) -> f32 {
		// SDF: f(p) = p.y - height(p.x, p.z)
		// Negative = below terrain (inside), Positive = above terrain (outside)
		let terrain_height = self.height_at(p.x, p.z);
		p.y - terrain_height
	}
}
