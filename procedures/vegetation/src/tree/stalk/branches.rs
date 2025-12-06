use sdf::{Sdf, CapsuleSdf};
use bevy::prelude::*;

/// A single branch segment SDF - a cylindrical segment
pub struct BranchSdf {
	/// Start point of the branch
	pub start: Vec3,
	/// End point of the branch
	pub end: Vec3,
	/// Radius of the branch (typically tapers, but constant for simplicity)
	pub radius: f32,
}

impl BranchSdf {
	pub fn new(start: Vec3, end: Vec3, radius: f32) -> Self {
		Self { start, end, radius }
	}
}

impl Sdf for BranchSdf {
	fn distance(&self, p: Vec3) -> f32 {
		// Use capsule SDF for a simple cylindrical branch
		let capsule = CapsuleSdf::new(self.start, self.end, self.radius);
		capsule.distance(p)
	}
}

