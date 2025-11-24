pub mod analysis;
pub mod combinators;
pub mod perlin_terrain;
pub mod region;
pub mod tetradhedron;
pub mod trapezoidal_prism;
pub mod tube;

pub use analysis::interval::{
	Sign, SignBoundary, SignUniformIntervals, SignUniformIntervalsIterator,
};
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

	fn sign_uniform_on_y(&self, _x: f32, _z: f32) -> SignUniformIntervals {
		SignUniformIntervals::default()
	}
}
