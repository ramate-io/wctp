pub mod boundary_mapping;
pub mod interval_mapping;

use crate::sdf::analysis::interval::{SignBoundary, SignUniformInterval};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignUniformIntervals {
	pub(crate) boundaries: BTreeSet<SignBoundary>,
}

impl SignUniformIntervals {
	/// Inserts a boundary into the intervals.
	pub fn insert_boundary(&mut self, boundary: SignBoundary) {
		self.boundaries.insert(boundary);
	}

	/// Inserts an interval into the intervals.
	pub fn insert_interval(&mut self, interval: SignUniformInterval) {
		self.boundaries.insert(interval.left);
		self.boundaries.insert(interval.right);
	}

	/// Queries a given open range for all of the different sign intervals that exist within it.
	///
	/// The current implementation is O(n) where n is the number of boundaries in the intervals.
	/// In theory, you could optimize with a binary. But, in most cases, the number of boundaries is small.
	pub fn in_range(&self, range: (f32, f32)) -> Vec<SignUniformInterval> {
		let mut intervals = Vec::new();
		for boundary in self.boundaries.iter() {
			if boundary.min >= range.0 && boundary.min < range.1 {
				intervals
					.push(SignUniformInterval { left: boundary.clone(), right: boundary.clone() });
			}
		}
		intervals
	}

	/// Computes the overlaps of the left interval set with the right interval set.
	pub fn overlaps_with(
		&self,
		other: &Self,
	) -> BTreeMap<Option<SignUniformInterval>, BTreeSet<SignUniformInterval>> {
		let mut all_overlaps = BTreeMap::new();

		let mut self_iter = self.clone().into_iter();
		let mut other_iter = other.clone().into_iter();

		while let Some(other_interval) = other_iter.next() {
			// Mark whether or not we've found an overlap with this other interval
			let mut overlap_exists = false;

			// Check all of the intervals in the self set for overlaps with the other interval
			// Note: later we may be able to optimize this with a binary search or other tracking.
			// But, in most cases, the number of intervals should be small enough that it doesn't really matter.
			while let Some(self_interval) = self_iter.next() {
				if self_interval.overlaps_with(&other_interval) {
					overlap_exists = true;
					// extend to all_overlaps for Some(self_interval) with other_interval
					all_overlaps
						.entry(Some(self_interval))
						.or_insert(BTreeSet::new())
						.insert(other_interval.clone());
				}
			}

			// If no overlap exists, map to the none key.
			if !overlap_exists {
				all_overlaps.entry(None).or_insert(BTreeSet::new()).insert(other_interval);
			}
		}

		all_overlaps
	}
}

pub struct SignUniformIntervalsIterator {
	intervals: Vec<SignBoundary>,
	index: usize,
	emitted_top: bool,
}

// Iterates left, right pairs beginning with the top constant, then through the members of the set, then ending with the bottom constant.
impl Iterator for SignUniformIntervalsIterator {
	type Item = SignUniformInterval;
	fn next(&mut self) -> Option<Self::Item> {
		if self.intervals.is_empty() {
			if !self.emitted_top {
				self.emitted_top = true;
				return Some(SignUniformInterval {
					left: SignBoundary::top(),
					right: SignBoundary::bottom(),
				});
			}
			return None;
		}

		if !self.emitted_top {
			self.emitted_top = true;
			// First pair: (top, first_element)
			return Some(SignUniformInterval {
				left: SignBoundary::top(),
				right: self.intervals[0].clone(),
			});
		}

		if self.index < self.intervals.len() - 1 {
			// Middle pairs: (elem_i, elem_{i+1})
			let left = self.intervals[self.index].clone();
			let right = self.intervals[self.index + 1].clone();
			self.index += 1;
			return Some(SignUniformInterval { left, right });
		}

		if self.index < self.intervals.len() {
			// Last pair: (last_element, bottom)
			let left = self.intervals[self.index].clone();
			self.index += 1;
			return Some(SignUniformInterval { left, right: SignBoundary::bottom() });
		}

		None
	}
}

impl IntoIterator for SignUniformIntervals {
	type Item = SignUniformInterval;
	type IntoIter = SignUniformIntervalsIterator;

	fn into_iter(self) -> Self::IntoIter {
		let intervals: Vec<SignBoundary> = self.boundaries.into_iter().collect();
		SignUniformIntervalsIterator { intervals, index: 0, emitted_top: false }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sdf::analysis::interval::Sign;

	#[test]
	fn test_empty_intervals() {
		let intervals = SignUniformIntervals::default();
		let pairs: Vec<_> = intervals.into_iter().collect();

		assert_eq!(pairs.len(), 1);
		assert_eq!(pairs[0].left.min, f32::NEG_INFINITY);
		assert_eq!(pairs[0].left.sign, Sign::Top);
		assert_eq!(pairs[0].right.min, f32::INFINITY);
		assert_eq!(pairs[0].right.sign, Sign::Bottom);
	}

	#[test]
	fn test_single_interval() {
		let mut intervals = SignUniformIntervals::default();
		intervals.insert_boundary(SignBoundary { min: 0.0, sign: Sign::Negative });

		let pairs: Vec<_> = intervals.into_iter().collect();

		assert_eq!(pairs.len(), 2);

		// First pair: (top, first_element)
		assert_eq!(pairs[0].left.min, f32::NEG_INFINITY);
		assert_eq!(pairs[0].left.sign, Sign::Top);
		assert_eq!(pairs[0].right.min, 0.0);
		assert_eq!(pairs[0].right.sign, Sign::Negative);

		// Second pair: (first_element, bottom)
		assert_eq!(pairs[1].left.min, 0.0);
		assert_eq!(pairs[1].left.sign, Sign::Negative);
		assert_eq!(pairs[1].right.min, f32::INFINITY);
		assert_eq!(pairs[1].right.sign, Sign::Bottom);
	}

	#[test]
	fn test_multiple_intervals() {
		let mut intervals = SignUniformIntervals::default();
		intervals.insert_boundary(SignBoundary { min: 0.0, sign: Sign::Negative });
		intervals.insert_boundary(SignBoundary { min: 5.0, sign: Sign::Positive });
		intervals.insert_boundary(SignBoundary { min: 10.0, sign: Sign::Negative });

		let pairs: Vec<_> = intervals.into_iter().collect();

		assert_eq!(pairs.len(), 4);

		// First pair: (top, first_element)
		assert_eq!(pairs[0].left.min, f32::NEG_INFINITY);
		assert_eq!(pairs[0].left.sign, Sign::Top);
		assert_eq!(pairs[0].right.min, 0.0);
		assert_eq!(pairs[0].right.sign, Sign::Negative);

		// Second pair: (0.0, 5.0)
		assert_eq!(pairs[1].left.min, 0.0);
		assert_eq!(pairs[1].left.sign, Sign::Negative);
		assert_eq!(pairs[1].right.min, 5.0);
		assert_eq!(pairs[1].right.sign, Sign::Positive);

		// Third pair: (5.0, 10.0)
		assert_eq!(pairs[2].left.min, 5.0);
		assert_eq!(pairs[2].left.sign, Sign::Positive);
		assert_eq!(pairs[2].right.min, 10.0);
		assert_eq!(pairs[2].right.sign, Sign::Negative);

		// Last pair: (last_element, bottom)
		assert_eq!(pairs[3].left.min, 10.0);
		assert_eq!(pairs[3].left.sign, Sign::Negative);
		assert_eq!(pairs[3].right.min, f32::INFINITY);
		assert_eq!(pairs[3].right.sign, Sign::Bottom);
	}

	#[test]
	fn test_interval_ordering() {
		let mut intervals = SignUniformIntervals::default();
		intervals.insert_boundary(SignBoundary { min: 10.0, sign: Sign::Positive });
		intervals.insert_boundary(SignBoundary { min: 0.0, sign: Sign::Negative });
		intervals.insert_boundary(SignBoundary { min: 5.0, sign: Sign::Positive });

		let pairs: Vec<_> = intervals.into_iter().collect();

		// Should be ordered by min value regardless of insertion order
		assert_eq!(pairs.len(), 4);
		assert_eq!(pairs[0].right.min, 0.0);
		assert_eq!(pairs[1].right.min, 5.0);
		assert_eq!(pairs[2].left.min, 5.0);
		assert_eq!(pairs[2].right.min, 10.0);
		assert_eq!(pairs[3].left.min, 10.0);
		assert_eq!(pairs[3].right.min, f32::INFINITY);
	}

	#[test]
	fn test_iterator_consumes() {
		let mut intervals = SignUniformIntervals::default();
		intervals.insert_boundary(SignBoundary { min: 0.0, sign: Sign::Negative });

		let mut iter = intervals.into_iter();
		assert!(iter.next().is_some());
		assert!(iter.next().is_some());
		assert!(iter.next().is_none());
	}
}
