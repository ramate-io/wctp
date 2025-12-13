use super::{simple::SimpleTrunkSegment, SegmentConfig};
use bevy::prelude::*;
use sdf::{RotateAlongRay, Sdf, Translate};

/// A join point where another segment can attach
#[derive(Debug, Clone)]
pub struct JoinPoint {
	/// Position in unit space (0-1 along the segment, 0 = bottom, 1 = top)
	pub unit_position: f32,
	/// Angle around the trunk axis in radians (0 = +X direction)
	pub angle: f32,
	/// Type of join point
	pub join_type: JoinType,
}

/// Type of join point
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinType {
	/// Join point for another trunk segment
	Trunk,
	/// Join point for a branch
	Branch,
}

/// SDF for a join point - a SimpleTrunkSegment angled in 3D space
/// The segment is oriented along a direction computed from the angle around the trunk
pub struct JoinPointSdf {
	/// The rotated and translated SimpleTrunkSegment
	sdf: Translate<RotateAlongRay<SimpleTrunkSegment>>,
}

impl JoinPointSdf {
	/// Create a join point SDF at the given position and angle
	///
	/// The join point is a SimpleTrunkSegment that:
	/// - Is positioned at unit_y along the trunk
	/// - Is angled outward from the trunk based on the angle parameter
	/// - Uses the provided config for its geometry
	///
	/// `upward_component` controls how much the segment angles upward:
	/// - 0.0 = horizontal (for branches)
	/// - 0.3 = slightly upward (for trunk splits)
	/// - 1.0 = vertical
	pub fn new(
		unit_y: f32,
		angle: f32,
		config: SegmentConfig,
		seed_offset: u32,
		upward_component: f32,
	) -> Self {
		// Create a SimpleTrunkSegment for the join point
		let mut join_config = config.clone();
		join_config.seed = config.seed + seed_offset;
		let join_segment = SimpleTrunkSegment::new(join_config);

		// Compute direction vector for the angled segment
		// The angle is around the Y axis, so we compute a direction in the XZ plane
		// and add an upward component
		let horizontal_dir = Vec3::new(angle.cos(), 0.0, angle.sin());

		// Mix horizontal direction with upward direction
		let direction =
			(horizontal_dir * (1.0 - upward_component) + Vec3::Y * upward_component).normalize();

		// Rotate the segment along this direction
		let rotated_segment = RotateAlongRay::new(join_segment, direction);

		// Position the segment at the join point
		// The segment's bottom (y=0) should be at unit_y
		let positioned_segment = Translate::new(rotated_segment, Vec3::new(0.0, unit_y, 0.0));

		Self { sdf: positioned_segment }
	}
}

impl Sdf for JoinPointSdf {
	fn distance(&self, p: Vec3) -> f32 {
		self.sdf.distance(p)
	}
}
