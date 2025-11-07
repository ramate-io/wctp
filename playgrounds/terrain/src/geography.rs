pub mod canyons;

use crate::terrain::TerrainConfig;
use bevy::prelude::*;

/// Trait for geographic features that modify terrain height
pub trait GeographicFeature {
	/// Modify the height at a given world position
	/// Returns the height modification (positive = raise, negative = lower)
	fn modify_height(
		&self,
		world_x: f32,
		world_z: f32,
		base_height: f32,
		config: &TerrainConfig,
	) -> f32;
}

/// Registry of geographic features to apply during terrain generation
#[derive(Resource)]
pub struct FeatureRegistry {
	pub features: Vec<Box<dyn GeographicFeature + Send + Sync>>,
}

impl Default for FeatureRegistry {
	fn default() -> Self {
		Self { features: Vec::new() }
	}
}

impl FeatureRegistry {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add_feature(&mut self, feature: Box<dyn GeographicFeature + Send + Sync>) {
		self.features.push(feature);
	}

	/// Apply all features to modify a height value
	pub fn apply_features(
		&self,
		world_x: f32,
		world_z: f32,
		base_height: f32,
		config: &TerrainConfig,
	) -> f32 {
		let mut height = base_height;
		for feature in &self.features {
			height += feature.modify_height(world_x, world_z, height, config);
		}
		height
	}
}
