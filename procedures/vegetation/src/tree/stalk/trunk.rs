use bevy::prelude::*;
use sdf::{Ellipse3d, Sdf};

/// A tree trunk SDF - a tapered cylinder from ground to canopy base
pub struct TrunkSdf {
	/// Base of the trunk (ground level)
	pub base: Vec3,
	/// Top of the trunk (where branches start)
	pub top: Vec3,
	/// Radius at the base
	pub base_radius: f32,
	/// Radius at the top (typically smaller than base)
	pub top_radius: f32,
}

impl TrunkSdf {
	pub fn new(base: Vec3, top: Vec3, base_radius: f32, top_radius: f32) -> Self {
		Self { base, top, base_radius, top_radius }
	}

	/// Helper: builds an orthonormal basis perpendicular to the trunk axis.
	fn orthonormal_basis(dir: Vec3) -> [Vec3; 2] {
		let right = if dir.x.abs() > dir.z.abs() {
			Vec3::new(-dir.y, dir.x, 0.0).normalize()
		} else {
			Vec3::new(0.0, -dir.z, dir.y).normalize()
		};
		let up = dir.cross(right).normalize();
		[right, up]
	}
}

impl Sdf for TrunkSdf {
	fn distance(&self, p: Vec3) -> f32 {
		let ray = self.top - self.base;
		let len = ray.length();
		if len < f32::EPSILON {
			// Degenerate case: base and top are the same
			return (p - self.base).length() - self.base_radius;
		}

		let dir = ray / len;
		let t = (p - self.base).dot(dir).clamp(0.0, len);
		let normalized_t = t / len;

		// Interpolate radius along the trunk
		let radius = self.base_radius * (1.0 - normalized_t) + self.top_radius * normalized_t;

		// Build orthonormal basis for cross-section
		let [right, up] = Self::orthonormal_basis(dir);
		let point_on_axis = self.base + dir * t;

		// Create ellipse for this cross-section (circular)
		let ellipse = Ellipse3d {
			center: point_on_axis,
			axes: [right, up],
			radii: Vec2::new(radius, radius),
		};

		// Distance to circular boundary
		let side_dist = ellipse.distance(p);

		// Distance along axis (for end caps)
		let cap_dist = (t - len * 0.5).abs() - len * 0.5;

		// Combine distances (capsule logic)
		let outside_dist = (side_dist.max(0.0).powi(2) + cap_dist.max(0.0).powi(2)).sqrt();
		let inside_dist = side_dist.max(cap_dist);

		if inside_dist < 0.0 {
			inside_dist
		} else {
			outside_dist
		}
	}
}
