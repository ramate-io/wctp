pub mod combinators;
pub mod inscribed_polygon;
pub mod perlin_terrain;

pub use combinators::{
	AddY, Difference, Elongate, Intersection, RotateY, Round, Scale, SmoothDifference,
	SmoothIntersection, SmoothUnion, Translate, Union,
};
pub use inscribed_polygon::InscribedPolygonSdf;
pub use perlin_terrain::{ElevationModulation, PerlinTerrainSdf};

use bevy::prelude::*;

/// Trait for Signed Distance Fields
/// Returns the signed distance from a point to the surface:
/// - Negative: inside/below the surface
/// - Zero: on the surface
/// - Positive: outside/above the surface
pub trait Sdf: Send + Sync {
	fn distance(&self, p: Vec3) -> f32;
}
