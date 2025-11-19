pub mod combinators;
pub mod perlin_terrain;
pub mod region;
pub mod tetradhedron;
pub mod trapezoidal_prism;
pub mod tube;

pub use combinators::{
	AddY, Difference, Elongate, Intersection, RotateY, Round, Scale, SmoothDifference,
	SmoothIntersection, SmoothUnion, Translate, Union,
};
pub use perlin_terrain::{ElevationModulation, PerlinTerrainSdf};
pub use tube::{Ellipse3d, TubeSdf};

use bevy::prelude::*;

/// Trait for Signed Distance Fields
/// Returns the signed distance from a point to the surface:
/// - Negative: inside/below the surface
/// - Zero: on the surface
/// - Positive: outside/above the surface
pub trait Sdf: Send + Sync {
	fn distance(&self, p: Vec3) -> f32;

	fn max_y_at(&self, x: f32, z: f32) -> f32 {
		// Binary search for the max y at the given x and z
		// Search range: from well below ground to well above max terrain height
		let mut min_y = -100.0;
		let mut max_y = 100.0;
		let epsilon = 0.01; // Precision threshold

		// Binary search for zero crossing (surface)
		for _ in 0..32 {
			// Limit iterations to prevent infinite loops
			if (max_y - min_y) < epsilon {
				break;
			}
			let mid_y = (min_y + max_y) * 0.5;
			let distance = self.distance(Vec3::new(x, mid_y, z));

			if distance < 0.0 {
				// Below surface, search higher
				min_y = mid_y;
			} else {
				// Above surface, search lower
				max_y = mid_y;
			}
		}

		// Return the surface height (where distance crosses zero)
		(min_y + max_y) * 0.5
	}
}
