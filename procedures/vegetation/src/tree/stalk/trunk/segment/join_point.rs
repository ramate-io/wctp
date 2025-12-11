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
