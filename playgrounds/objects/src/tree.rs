use bevy::prelude::*;
use sdf::Sdf;
use vegetation_sdf::tree::{CanopySdf, TrunkSdf};

/// A single tree SDF combining trunk and canopy
pub struct TreeSdf {
	trunk: TrunkSdf,
	canopy: CanopySdf,
}

impl TreeSdf {
	pub fn new(
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

/// Create a single tree SDF configured for kilometers
/// Tree is placed at origin (0, 0, 0)
pub fn create_tree_sdf() -> TreeSdf {
	// Tree parameters in kilometers
	let base = Vec3::ZERO;
	let trunk_height = 0.005; // 5 meters
	let trunk_base_radius = 0.0005; // 0.5 meters
	let trunk_top_radius = trunk_base_radius * 0.6; // Tapered
	let canopy_radius = 0.003; // 3 meters
	let canopy_height = canopy_radius * 0.8; // Slightly flattened

	TreeSdf::new(
		base,
		trunk_height,
		trunk_base_radius,
		trunk_top_radius,
		canopy_radius,
		canopy_height,
	)
}
