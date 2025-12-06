use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use sdf::Sdf;

use crate::tree::{CanopySdf, TrunkSdf};

/// Configuration for forest generation
pub struct ForestConfig {
	/// Seed for Perlin noise
	pub seed: u32,
	/// Bounds of the forest (min_x, max_x, min_z, max_z)
	pub bounds: (f32, f32, f32, f32),
	/// Grid spacing for tree placement (smaller = denser)
	pub grid_spacing: f32,
	/// Threshold for placing trees (0.0 to 1.0, higher = fewer trees)
	/// Trees are placed where noise value is above this threshold
	pub placement_threshold: f32,
	/// Frequency of the placement noise
	pub noise_frequency: f32,
	/// Base height for tree placement (Y coordinate)
	pub base_height: f32,
	/// Trunk height range (min, max)
	pub trunk_height: (f32, f32),
	/// Trunk base radius range (min, max)
	pub trunk_base_radius: (f32, f32),
	/// Trunk top radius as fraction of base radius
	pub trunk_taper: f32,
	/// Canopy radius range (min, max) - typically larger than trunk
	pub canopy_radius: (f32, f32),
	/// Canopy height (vertical radius) as fraction of horizontal radius
	pub canopy_height_ratio: f32,
}

impl Default for ForestConfig {
	fn default() -> Self {
		Self {
			seed: 42,
			bounds: (-50.0, 50.0, -50.0, 50.0),
			grid_spacing: 5.0,
			placement_threshold: 0.3,
			noise_frequency: 0.1,
			base_height: 0.0,
			trunk_height: (3.0, 8.0),
			trunk_base_radius: (0.3, 0.6),
			trunk_taper: 0.6,
			canopy_radius: (2.0, 4.0),
			canopy_height_ratio: 0.8,
		}
	}
}

/// A single tree SDF combining trunk and canopy
struct TreeSdf {
	trunk: TrunkSdf,
	canopy: CanopySdf,
}

impl TreeSdf {
	fn new(
		base: Vec3,
		trunk_height: f32,
		trunk_base_radius: f32,
		trunk_top_radius: f32,
		canopy_radius: f32,
		canopy_height: f32,
	) -> Self {
		let top = base + Vec3::Y * trunk_height;
		let trunk = TrunkSdf::new(base, top, trunk_base_radius, trunk_top_radius);

		// Place canopy above trunk, centered at top
		let canopy_center = top + Vec3::Y * canopy_height * 0.3; // Slightly above trunk top
		let canopy =
			CanopySdf::new(canopy_center, Vec3::new(canopy_radius, canopy_height, canopy_radius));

		Self { trunk, canopy }
	}
}

impl Sdf for TreeSdf {
	fn distance(&self, p: Vec3) -> f32 {
		// Union of trunk and canopy
		let trunk_dist = self.trunk.distance(p);
		let canopy_dist = self.canopy.distance(p);
		trunk_dist.min(canopy_dist)
	}
}

/// A forest SDF - multiple trees placed using Perlin noise
pub struct ForestSdf {
	trees: Vec<TreeSdf>,
	// Store config bounds for sign_uniform_on_y optimization
	config_bounds: (f32, f32, f32, f32), // (min_x, max_x, min_z, max_z)
}

impl ForestSdf {
	/// Generate a forest from the given configuration
	pub fn new(config: ForestConfig) -> Self {
		let mut trees = Vec::new();
		let noise = Perlin::new(config.seed);

		// Generate trees on a grid
		let mut x = config.bounds.0;
		while x <= config.bounds.1 {
			let mut z = config.bounds.2;
			while z <= config.bounds.3 {
				// Sample noise to decide if we place a tree here
				let noise_value = noise.get([
					x as f64 * config.noise_frequency as f64,
					z as f64 * config.noise_frequency as f64,
				]) as f32;
				let normalized_noise = (noise_value + 1.0) * 0.5; // Convert from [-1, 1] to [0, 1]

				if normalized_noise > config.placement_threshold {
					// Use noise to vary tree parameters
					let height_noise = noise.get([x as f64 * 0.15, z as f64 * 0.15]) as f32;
					let radius_noise = noise.get([x as f64 * 0.2, z as f64 * 0.2]) as f32;

					// Normalize noise values to [0, 1]
					let h_norm = (height_noise + 1.0) * 0.5;
					let r_norm = (radius_noise + 1.0) * 0.5;

					// Calculate tree parameters
					let trunk_height = config.trunk_height.0
						+ (config.trunk_height.1 - config.trunk_height.0) * h_norm;
					let trunk_base_radius = config.trunk_base_radius.0
						+ (config.trunk_base_radius.1 - config.trunk_base_radius.0) * r_norm;
					let trunk_top_radius = trunk_base_radius * config.trunk_taper;

					let canopy_radius = config.canopy_radius.0
						+ (config.canopy_radius.1 - config.canopy_radius.0) * r_norm;
					let canopy_height = canopy_radius * config.canopy_height_ratio;

					let base = Vec3::new(x, config.base_height, z);

					trees.push(TreeSdf::new(
						base,
						trunk_height,
						trunk_base_radius,
						trunk_top_radius,
						canopy_radius,
						canopy_height,
					));
				}

				z += config.grid_spacing;
			}
			x += config.grid_spacing;
		}

		Self { trees, config_bounds: config.bounds }
	}
}

impl Sdf for ForestSdf {
	fn distance(&self, p: Vec3) -> f32 {
		if self.trees.is_empty() {
			return f32::MAX;
		}

		// Union all trees - find minimum distance
		let mut min_dist = f32::MAX;
		for tree in &self.trees {
			let dist = tree.distance(p);
			min_dist = min_dist.min(dist);
		}

		min_dist
	}

	fn sign_uniform_on_y(&self, x: f32, z: f32) -> sdf::SignUniformIntervals {
		// uniformly positive outside the forest bounds
		// if in bounds
		if x > self.config_bounds.0
			&& x < self.config_bounds.1
			&& z > self.config_bounds.2
			&& z < self.config_bounds.3
		{
			// uniformly positive inside the forest bounds
			let mut intervals = sdf::SignUniformIntervals::default();
			intervals.insert_boundary(sdf::SignBoundary {
				min: f32::NEG_INFINITY,
				sign: sdf::Sign::Positive,
			});
			intervals.insert_boundary(sdf::SignBoundary {
				min: f32::INFINITY,
				sign: sdf::Sign::Positive,
			});
			intervals
		} else {
			sdf::SignUniformIntervals::default()
		}
	}
}
