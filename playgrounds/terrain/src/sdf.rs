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
	/// The sign is unknown.
	Top,
	/// The sign is negative
	Negative,
	/// The sign is positive
	Positive,
	/// The sign is known but undefined.
	Bottom,
}

/// The sign is uniform from the min to some next boundary which will be placed in the intervals.
#[derive(Debug, Clone)]

pub struct SignUniform {
	pub min: f32,
	pub sign: Sign,
}

impl SignUniform {
	/// The sign is uniformly unknown from negative infinity.
	pub const fn top() -> Self {
		Self { min: f32::NEG_INFINITY, sign: Sign::Top }
	}

	/// The sign is uniformly undefined from positive infinity.
	pub const fn bottom() -> Self {
		Self { min: f32::INFINITY, sign: Sign::Bottom }
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

#[derive(Debug, Clone, Default)]
pub struct SignUniformIntervals {
	pub intervals: BTreeSet<SignUniform>,
}

impl SignUniformIntervals {}

pub struct SignUniformIntervalsIterator {
	intervals: Vec<SignUniform>,
	index: usize,
	emitted_top: bool,
}

// Iterates left, right pairs beginning with the top constant, then through the members of the set, then ending with the bottom constant.
impl Iterator for SignUniformIntervalsIterator {
	type Item = (SignUniform, SignUniform);
	fn next(&mut self) -> Option<Self::Item> {
		if self.intervals.is_empty() {
			if !self.emitted_top {
				self.emitted_top = true;
				return Some((SignUniform::top(), SignUniform::bottom()));
			}
			return None;
		}

		if !self.emitted_top {
			self.emitted_top = true;
			// First pair: (top, first_element)
			return Some((SignUniform::top(), self.intervals[0].clone()));
		}

		if self.index < self.intervals.len() - 1 {
			// Middle pairs: (elem_i, elem_{i+1})
			let left = self.intervals[self.index].clone();
			let right = self.intervals[self.index + 1].clone();
			self.index += 1;
			return Some((left, right));
		}

		if self.index < self.intervals.len() {
			// Last pair: (last_element, bottom)
			let left = self.intervals[self.index].clone();
			self.index += 1;
			return Some((left, SignUniform::bottom()));
		}

		None
	}
}

impl IntoIterator for SignUniformIntervals {
	type Item = (SignUniform, SignUniform);
	type IntoIter = SignUniformIntervalsIterator;

	fn into_iter(self) -> Self::IntoIter {
		let intervals: Vec<SignUniform> = self.intervals.into_iter().collect();
		SignUniformIntervalsIterator { intervals, index: 0, emitted_top: false }
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
		SignUniformIntervals::default()
	}
}
