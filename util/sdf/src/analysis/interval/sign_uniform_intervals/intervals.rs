pub mod boundary_mapping;
pub mod interval_interator;
pub mod interval_mapping;

use crate::analysis::interval::{SignBoundary, SignUniformInterval};
use interval_interator::SignUniformIntervalsIterator;
use std::collections::BTreeSet;

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
}

impl IntoIterator for SignUniformIntervals {
	type Item = SignUniformInterval;
	type IntoIter = SignUniformIntervalsIterator;

	fn into_iter(self) -> Self::IntoIter {
		let intervals: Vec<SignBoundary> = self.boundaries.into_iter().collect();
		SignUniformIntervalsIterator::new(intervals)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::analysis::interval::Sign;

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
