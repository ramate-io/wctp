use bevy::prelude::*;
use sdf::Sdf;
use vegetation_sdf::tree::stalk::trunk::segment::{simple::SimpleTrunkSegment, SegmentConfig};
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

/// Wrapper SDF for a trunk segment that transforms world space to unit space
/// The segment works in unit space (0-1 for y, centered at origin for x/z)
/// This wrapper scales it to world space (kilometers)
pub struct SegmentSdf {
	segment: SimpleTrunkSegment,
	/// Scale factor to convert unit space to world space (km)
	/// Segment height in world space = scale
	scale: f32,
}

impl SegmentSdf {
	pub fn new(segment: SimpleTrunkSegment, scale: f32) -> Self {
		Self { segment, scale }
	}
}

impl Sdf for SegmentSdf {
	fn distance(&self, p: Vec3) -> f32 {
		// Transform world space to unit space
		// In unit space: y is 0-1, x/z are centered at origin
		// We'll scale x/z by the same factor as y to maintain proportions
		let unit_p = Vec3::new(p.x / self.scale, p.y / self.scale, p.z / self.scale);

		// Get distance in unit space and scale back to world space
		self.segment.distance(unit_p) * self.scale
	}
}

/// Create a simple segment SDF at the origin
pub fn create_segment_sdf() -> SegmentSdf {
	let config = SegmentConfig {
		seed: 42,
		base_radius: 0.5,
		top_radius: 0.4,
		noise_amplitude: 0.05,
		noise_frequency: 5.0,
	};
	let segment = SimpleTrunkSegment::new(config);

	// Scale to 0.01 km (10 meters) height
	let scale = 0.01;

	SegmentSdf::new(segment, scale)
}
