pub mod analysis;
pub mod combinators;
pub mod perlin_terrain;
pub mod region;
pub mod tetradhedron;
pub mod trapezoidal_prism;
pub mod tube;

pub use analysis::bounds::Bounds;
pub use analysis::interval::{
	Sign, SignBoundary, SignUniformInterval, SignUniformIntervals, SignUniformIntervalsIterator,
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

	/// Returns the bounds of the SDF, i.e., the region over which the SDF is defined.
	/// This can form pessimistic boundaries for analysis of the SDF.
	///
	/// For example, when unioning two SDFs, you can say everything within the bounds of either SDF
	/// is Top. Everything outside the intesecting bounds can take the value of which SDF is defined.
	fn bounds(&self) -> Bounds {
		Bounds::Unbounded
	}
}
