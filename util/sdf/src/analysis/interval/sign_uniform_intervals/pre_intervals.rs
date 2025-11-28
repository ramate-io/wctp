use crate::analysis::interval::{SignBoundary, SignUniformInterval, SignUniformIntervals};
use std::collections::BTreeSet;

/// A collection of unnormalized boundaries
/// This is the constructor API for [SignUniformIntervals].
#[derive(Debug, Clone, Default)]
pub struct PreSignUniformIntervals {
	unnormalized_boundaries: BTreeSet<SignBoundary>,
}

impl PreSignUniformIntervals {
	pub fn new() -> Self {
		Self { unnormalized_boundaries: BTreeSet::new() }
	}

	/// Inserts a boundary into the intervals.
	pub fn insert_boundary(&mut self, boundary: SignBoundary) {
		self.unnormalized_boundaries.insert(boundary);
	}

	/// Inserts an interval into the intervals.
	pub fn insert_interval(&mut self, interval: SignUniformInterval) {
		self.unnormalized_boundaries.insert(interval.left);
		self.unnormalized_boundaries.insert(interval.right);
	}

	/// Normalizes the intervals and computes the [SignUniformIntervals].
	pub fn normalize(self) -> SignUniformIntervals {
		let mut normalized_boundaries = BTreeSet::new();
		let mut previous_boundary: Option<SignBoundary> = None;
		for boundary in self.unnormalized_boundaries.iter() {
			if let Some(previous_boundary) = previous_boundary {
				if previous_boundary.sign != boundary.sign {
					normalized_boundaries.insert(boundary.clone());
				}
			} else {
				normalized_boundaries.insert(boundary.clone());
			}
			previous_boundary = Some(boundary.clone());
		}

		// Top and Bottom are canonical boundaries that are always present.
		normalized_boundaries.insert(SignBoundary::top());
		normalized_boundaries.insert(SignBoundary::bottom());

		SignUniformIntervals { boundaries: normalized_boundaries }
	}
}
