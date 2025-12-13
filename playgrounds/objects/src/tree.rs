use bevy::prelude::*;
use noise::NoiseFn;
use sdf::Sdf;
use vegetation_sdf::tree::stalk::trunk::segment::{
	simple::SimpleTrunkSegment, trunk_split::TrunkSplitSegment, SegmentConfig,
};
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
	/// Translation of the segment in world space
	translation: Vec3,
	/// Rotation of the segment in world space
	rotation: Quat,
}

impl SegmentSdf {
	pub fn new(segment: SimpleTrunkSegment, scale: f32) -> Self {
		Self { segment, scale, translation: Vec3::ZERO, rotation: Quat::IDENTITY }
	}

	pub fn with_translation(self, translation: Vec3) -> Self {
		Self { translation, ..self }
	}

	pub fn with_rotation(self, rotation: Quat) -> Self {
		Self { rotation, ..self }
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

	fn translation(&self) -> Vec3 {
		self.translation
	}

	fn rotation(&self) -> Quat {
		self.rotation
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

/// Wrapper types for each segment instance to allow separate Bevy resource management
pub struct BaseSegmentSdf(pub SegmentSdf);
pub struct SplitSegment1Sdf(pub SegmentSdf);
pub struct SplitSegment2Sdf(pub SegmentSdf);

impl Sdf for BaseSegmentSdf {
	fn distance(&self, p: Vec3) -> f32 {
		self.0.distance(p)
	}

	fn translation(&self) -> Vec3 {
		self.0.translation()
	}

	fn rotation(&self) -> Quat {
		self.0.rotation()
	}

	fn scale(&self) -> Vec3 {
		self.0.scale()
	}
}

impl Sdf for SplitSegment1Sdf {
	fn distance(&self, p: Vec3) -> f32 {
		self.0.distance(p)
	}

	fn translation(&self) -> Vec3 {
		self.0.translation()
	}

	fn rotation(&self) -> Quat {
		self.0.rotation()
	}

	fn scale(&self) -> Vec3 {
		self.0.scale()
	}
}

impl Sdf for SplitSegment2Sdf {
	fn distance(&self, p: Vec3) -> f32 {
		self.0.distance(p)
	}

	fn translation(&self) -> Vec3 {
		self.0.translation()
	}

	fn rotation(&self) -> Quat {
		self.0.rotation()
	}

	fn scale(&self) -> Vec3 {
		self.0.scale()
	}
}

/// Create three separate segments for mesh composition: base + 2 splits
/// Returns (base_segment, split_segment_1, split_segment_2) as wrapper types
pub fn create_trunk_split_segments() -> (BaseSegmentSdf, SplitSegment1Sdf, SplitSegment2Sdf) {
	let config = SegmentConfig {
		seed: 42,
		base_radius: 0.5,
		top_radius: 0.4,
		noise_amplitude: 0.05,
		noise_frequency: 5.0,
	};

	// Scale to 0.01 km (10 meters) height
	let scale = 0.01;

	// Base segment at origin
	let base_segment = SimpleTrunkSegment::new(config.clone());
	let base_sdf = SegmentSdf::new(base_segment, scale);

	// Generate split positions using the same logic as TrunkSplitSegment
	let num_splits = 2;
	let split_seed_offset = 2000;
	let split_noise = noise::Perlin::new(config.seed + split_seed_offset);

	let mut split_segments = Vec::new();

	for i in 0..num_splits {
		// All splits are slightly below the top (0.95 instead of 1.0)
		let unit_position = 0.95;

		// Use noise to determine angle around trunk, evenly distributed
		let base_angle = (i as f32 / num_splits as f32) * 2.0 * std::f32::consts::PI;
		let angle_noise = split_noise.get([i as f64 * 0.3, 0.0]) as f32;
		let _angle = base_angle + angle_noise * 0.2;

		// Create segment for this split
		let mut split_config = config.clone();
		split_config.seed = config.seed + split_seed_offset + i as u32;
		let split_segment = SimpleTrunkSegment::new(split_config);

		// Compute direction vector for the angled segment (commented out for now)
		// The angle is around the Y axis, so we compute a direction in the XZ plane
		// and add an upward component for trunk splits
		// let horizontal_dir = Vec3::new(angle.cos(), 0.0, angle.sin());

		// For trunk splits, angle upward and outward
		// Mix horizontal direction with upward direction
		// let upward_component = 0.3; // How much to angle upward (0 = horizontal, 1 = vertical)
		// let direction =
		// 	(horizontal_dir * (1.0 - upward_component) + Vec3::Y * upward_component).normalize();

		// Compute rotation from Y axis to direction (commented out for now)
		// let y_axis = Vec3::Y;
		// let rotation = if direction.dot(y_axis) > 0.999 {
		// 	Quat::IDENTITY
		// } else if direction.dot(y_axis) < -0.999 {
		// 	Quat::from_rotation_x(std::f32::consts::PI)
		// } else {
		// 	Quat::from_rotation_arc(y_axis, direction)
		// };

		// Position at join point (unit_position * scale in world space)
		// The segment's bottom (y=0) should be at this position
		let translation = Vec3::new(0.0, unit_position * scale, 0.0);

		let split_sdf = SegmentSdf::new(split_segment, scale).with_translation(translation);
		// .with_rotation(rotation);
		split_segments.push(split_sdf);
	}

	// Return the segments as wrapper types
	(
		BaseSegmentSdf(base_sdf),
		SplitSegment1Sdf(split_segments.remove(0)),
		SplitSegment2Sdf(split_segments.remove(0)),
	)
}

/// Wrapper SDF for a trunk split segment that transforms world space to unit space
/// The segment works in unit space (0-1 for y, centered at origin for x/z)
/// This wrapper scales it to world space (kilometers)
pub struct TrunkSplitSegmentSdf {
	segment: TrunkSplitSegment,
	/// Scale factor to convert unit space to world space (km)
	/// Segment height in world space = scale
	scale: f32,
}

impl TrunkSplitSegmentSdf {
	pub fn new(segment: TrunkSplitSegment, scale: f32) -> Self {
		Self { segment, scale }
	}
}

impl Sdf for TrunkSplitSegmentSdf {
	fn distance(&self, p: Vec3) -> f32 {
		// Transform world space to unit space
		// In unit space: y is 0-1, x/z are centered at origin
		// We'll scale x/z by the same factor as y to maintain proportions
		let unit_p = Vec3::new(p.x / self.scale, p.y / self.scale, p.z / self.scale);

		// Get distance in unit space and scale back to world space
		self.segment.distance(unit_p) * self.scale
	}
}

/// Create a trunk split segment SDF at the origin
pub fn create_trunk_split_segment_sdf() -> TrunkSplitSegmentSdf {
	let config = SegmentConfig {
		seed: 42,
		base_radius: 0.5,
		top_radius: 0.4,
		noise_amplitude: 0.05,
		noise_frequency: 5.0,
	};
	let segment = TrunkSplitSegment::new(config, 3); // 3 splits

	// Scale to 0.01 km (10 meters) height
	let scale = 0.01;

	TrunkSplitSegmentSdf::new(segment, scale)
}
