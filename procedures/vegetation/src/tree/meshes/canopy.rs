pub mod branch;

use bevy::prelude::*;
use sdf::{EllipsoidSdf, Sdf};

/// A tree canopy SDF - the foliage volume above the trunk
/// Can be represented as an ellipsoid, sphere, or union of multiple volumes
pub struct CanopySdf {
	/// Center of the canopy
	pub center: Vec3,
	/// Radii for ellipsoid shape (x, y, z)
	/// Typically wider in x/z than in y (height)
	pub radii: Vec3,
}

impl CanopySdf {
	pub fn new(center: Vec3, radii: Vec3) -> Self {
		Self { center, radii }
	}

	/// Create a spherical canopy
	pub fn spherical(center: Vec3, radius: f32) -> Self {
		Self { center, radii: Vec3::splat(radius) }
	}
}

impl Sdf for CanopySdf {
	fn distance(&self, p: Vec3) -> f32 {
		// Use ellipsoid SDF for the canopy shape
		// If all radii are equal, it's effectively a sphere
		let ellipsoid = EllipsoidSdf::new(self.center, self.radii);
		ellipsoid.distance(p)
	}
}
