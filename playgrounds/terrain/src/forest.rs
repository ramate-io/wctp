use bevy::prelude::*;
use sdf::Sdf;
use vegetation_sdf::{ForestConfig, ForestSdf};

/// Wrapper for ForestSdf that implements Sdf trait
/// This allows it to be used with the chunk manager
pub struct ForestSdfWrapper {
	forest: ForestSdf,
}

impl ForestSdfWrapper {
	pub fn new(config: ForestConfig) -> Self {
		Self { forest: ForestSdf::new(config) }
	}
}

impl Sdf for ForestSdfWrapper {
	fn distance(&self, p: Vec3) -> f32 {
		self.forest.distance(p)
	}
}

/// Create a forest SDF configured for kilometers, floating at 5km height
pub fn create_forest_sdf(seed: u32) -> ForestSdfWrapper {
	let config = ForestConfig {
		seed,
		// Bounds: 50km x 50km area
		bounds: (-25.0, 25.0, -25.0, 25.0),
		// Grid spacing: 0.5km (500m) - reasonable tree spacing
		grid_spacing: 0.5,
		placement_threshold: 0.3,
		noise_frequency: 0.1,
		// Base height: 5km above ground
		base_height: 5.0,
		// Trunk height: 3-8 meters = 0.003-0.008 km
		trunk_height: (0.003, 0.008),
		// Trunk base radius: 0.3-0.6 meters = 0.0003-0.0006 km
		trunk_base_radius: (0.0003, 0.0006),
		trunk_taper: 0.6,
		// Canopy radius: 2-4 meters = 0.002-0.004 km
		canopy_radius: (0.002, 0.004),
		canopy_height_ratio: 0.8,
	};

	ForestSdfWrapper::new(config)
}
