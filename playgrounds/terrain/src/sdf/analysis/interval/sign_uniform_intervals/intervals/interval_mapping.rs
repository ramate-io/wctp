pub mod combinators;

use crate::sdf::analysis::interval::{SignUniformInterval, SignUniformIntervals};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntervalMapping {
	mapping: BTreeMap<Option<SignUniformInterval>, BTreeSet<SignUniformInterval>>,
}

impl IntervalMapping {
	pub fn into_iter(
		self,
	) -> impl Iterator<Item = (Option<SignUniformInterval>, BTreeSet<SignUniformInterval>)> {
		self.mapping.into_iter()
	}

	#[cfg(test)]
	pub fn new() -> Self {
		Self { mapping: BTreeMap::new() }
	}

	#[cfg(test)]
	pub fn map(&mut self, from: Option<SignUniformInterval>, to: SignUniformInterval) {
		self.mapping.entry(from).or_insert(BTreeSet::new()).insert(to);
	}
}

impl SignUniformIntervals {
	/// Computes the overlaps of the left interval set with the right interval set.
	pub fn interval_mapping(&self, other: &Self) -> IntervalMapping {
		let mut all_intersections = BTreeMap::new();

		let mut other_iter = other.clone().into_iter();

		while let Some(other_interval) = other_iter.next() {
			// Mark whether or not we've found an overlap with this other interval
			let mut intersection_exists = false;
			let mut self_iter = self.clone().into_iter();

			// Check all of the intervals in the self set for overlaps with the other interval
			// Note: later we may be able to optimize this with a binary search or other tracking.
			// But, in most cases, the number of intervals should be small enough that it doesn't really matter.
			while let Some(self_interval) = self_iter.next() {
				if self_interval.intersects_with(&other_interval) {
					intersection_exists = true;
					// extend to all_intersections for Some(self_interval) with other_interval
					all_intersections
						.entry(Some(self_interval))
						.or_insert(BTreeSet::new())
						.insert(other_interval.clone());
				}
			}

			// If no overlap exists, map to the none key.
			if !intersection_exists {
				all_intersections.entry(None).or_insert(BTreeSet::new()).insert(other_interval);
			}
		}

		IntervalMapping { mapping: all_intersections }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sdf::analysis::interval::{PreSignUniformIntervals, Sign, SignBoundary};

	#[test]
	fn test_interval_mapping() {
		// left intervals
		let a_0 = SignBoundary { min: 0.0, sign: Sign::Negative };
		let a_1 = SignBoundary { min: 1.0, sign: Sign::Positive };
		let a_2 = SignBoundary { min: 2.0, sign: Sign::Negative };

		// right intervals
		let b_0 = SignBoundary { min: 1.0, sign: Sign::Negative };
		let b_1 = SignBoundary { min: 2.0, sign: Sign::Positive };
		let b_2 = SignBoundary { min: 3.0, sign: Sign::Negative };

		let mut left_pre_intervals = PreSignUniformIntervals::new();
		left_pre_intervals.insert_boundary(a_0.clone());
		left_pre_intervals.insert_boundary(a_1.clone());
		left_pre_intervals.insert_boundary(a_2.clone());
		let left_intervals = left_pre_intervals.normalize();

		let mut right_pre_intervals = PreSignUniformIntervals::new();
		right_pre_intervals.insert_boundary(b_0.clone());
		right_pre_intervals.insert_boundary(b_1.clone());
		right_pre_intervals.insert_boundary(b_2.clone());
		let right_intervals = right_pre_intervals.normalize();

		let interval_mapping = left_intervals.interval_mapping(&right_intervals);

		let mut reference_mapping = IntervalMapping::new();
		// top, a_0 -> top, b_0
		reference_mapping.map(
			Some(SignUniformInterval { left: SignBoundary::top(), right: a_0.clone() }),
			SignUniformInterval {
				left: SignBoundary { min: f32::NEG_INFINITY, sign: Sign::Top },
				right: b_0.clone(),
			},
		);

		// a_0, a_1 -> top, b_0
		reference_mapping.map(
			Some(SignUniformInterval { left: a_0.clone(), right: a_1.clone() }),
			SignUniformInterval { left: SignBoundary::top(), right: b_0.clone() },
		);

		// a_1, a_2 -> b_0, b_1
		reference_mapping.map(
			Some(SignUniformInterval { left: a_1.clone(), right: a_2.clone() }),
			SignUniformInterval { left: b_0.clone(), right: b_1.clone() },
		);

		// a_2, bottom -> b_1, b_2
		reference_mapping.map(
			Some(SignUniformInterval { left: a_2.clone(), right: SignBoundary::bottom() }),
			SignUniformInterval { left: b_1.clone(), right: b_2.clone() },
		);

		// a_2, bottom -> b_2, bottom
		reference_mapping.map(
			Some(SignUniformInterval { left: a_2.clone(), right: SignBoundary::bottom() }),
			SignUniformInterval { left: b_2.clone(), right: SignBoundary::bottom() },
		);

		// None -> top, top
		reference_mapping.map(
			None,
			SignUniformInterval { left: SignBoundary::top(), right: SignBoundary::top() },
		);

		// None -> bottom, bottom
		reference_mapping.map(
			None,
			SignUniformInterval { left: SignBoundary::bottom(), right: SignBoundary::bottom() },
		);

		assert_eq!(interval_mapping, reference_mapping);
	}
}
