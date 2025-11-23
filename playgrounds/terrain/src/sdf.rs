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
use std::collections::BTreeSet;
pub use tube::{Ellipse3d, TubeSdf};

use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sign {
	/// The sign is unknown
	Top,
	/// The sign is negative
	Negative,
	/// The sign is positive
	Positive,
}

/// The sign is uniform from the min to some next boundary which will be placed in the intervals.
#[derive(Debug, Clone)]

pub struct SignUniform {
	pub min: f32,
	pub sign: Sign,
}

impl SignUniform {
	/// The sign is uniformly unknown from negative infinity.
	pub fn top() -> Self {
		Self { min: f32::NEG_INFINITY, sign: Sign::Top }
	}
}

impl PartialOrd for SignUniform {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// compare min then sign
		Some(
			self.min
				.partial_cmp(&other.min)
				.unwrap_or(std::cmp::Ordering::Equal)
				.then_with(|| {
					self.sign.partial_cmp(&other.sign).unwrap_or(std::cmp::Ordering::Equal)
				}),
		)
	}
}

impl PartialEq for SignUniform {
	fn eq(&self, other: &Self) -> bool {
		self.min == other.min && self.sign == other.sign
	}
}

impl Eq for SignUniform {}

impl Ord for SignUniform {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.partial_cmp(other).unwrap()
	}
}

#[derive(Debug, Clone)]
pub struct SignUniformIntervals {
	pub intervals: BTreeSet<SignUniform>,
}

impl SignUniformIntervals {
	pub fn top() -> Self {
		let mut intervals = BTreeSet::new();
		intervals.insert(SignUniform::top());
		Self { intervals }
	}
}

impl Iterator for SignUniformIntervals {
	type Item = ();
	fn next(&mut self) -> Option<Self::Item> {
		self.intervals.pop_first()
	}
}

/// Trait for Signed Distance Fields
/// Returns the signed distance from a point to the surface:
/// - Negative: inside/below the surface
/// - Zero: on the surface
/// - Positive: outside/above the surface
pub trait Sdf: Send + Sync {
	fn distance(&self, p: Vec3) -> f32;

	fn sign_uniform_on_y(&self, _x: f32, _z: f32) -> SignUniformIntervals {
		SignUniformIntervals::top()
	}
}
