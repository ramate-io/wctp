pub mod combinators;

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sign {
	/// The sign is unknown.
	Top,
	/// The sign is negative.
	Negative,
	/// The sign is positive.
	Positive,
	/// The sign is known but undefined.
	Bottom,
}

impl Sign {
	/// Returns true if the sign is well behaved.
	pub fn is_well_behaved(&self) -> bool {
		matches!(self, Sign::Negative | Sign::Positive)
	}

	/// Returns true if sign is negative
	pub fn is_negative(&self) -> bool {
		matches!(self, Sign::Negative)
	}

	/// Returns true if sign is negative
	pub fn is_positive(&self) -> bool {
		matches!(self, Sign::Positive)
	}

	/// Returns the union of the two signs.
	pub fn union(&self, other: &Self) -> Self {
		match (self, other) {
			(Sign::Negative, _) => Sign::Negative,
			(_, Sign::Negative) => Sign::Negative,
			(Sign::Positive, Sign::Positive) => Sign::Positive,
			_ => Sign::Top,
		}
	}

	/// Flips the sign if negative, otherwise returns the sign.
	pub fn flip(&self) -> Self {
		if self == &Sign::Negative {
			Sign::Positive
		} else {
			self.clone()
		}
	}

	/// Returns the difference of the two signs.
	pub fn difference(&self, other: &Self) -> Self {
		match (self, other) {
			// whatever the self sign is, if the other is negative, then the result is positive
			(_, Sign::Negative) => Sign::Positive,
			// otherwise, the sign stays the same
			_ => self.clone(),
		}
	}
}

/// The sign is uniform from the min to some next boundary which will be placed in the intervals.
#[derive(Debug, Clone)]
pub struct SignBoundary {
	pub min: f32,
	pub sign: Sign,
}

impl SignBoundary {
	/// The sign is uniformly unknown from negative infinity.
	pub const fn top() -> Self {
		Self { min: f32::NEG_INFINITY, sign: Sign::Top }
	}

	/// The sign is uniformly undefined from positive infinity.
	pub const fn bottom() -> Self {
		Self { min: f32::INFINITY, sign: Sign::Bottom }
	}

	/// Constructrs a version of the boundary with the sign flipped.
	pub fn flip(&self) -> Self {
		Self { min: self.min, sign: self.sign.flip() }
	}
}

impl PartialOrd for SignBoundary {
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

impl PartialEq for SignBoundary {
	fn eq(&self, other: &Self) -> bool {
		self.min == other.min && self.sign == other.sign
	}
}

impl Eq for SignBoundary {}

impl Ord for SignBoundary {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.partial_cmp(other).unwrap()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignUniformInterval {
	pub left: SignBoundary,
	pub right: SignBoundary,
}

impl SignUniformInterval {
	/// Gets the open range of the sign uniform interval.
	///
	/// All intervals are on an open range; the naming is to make it clear on each call.
	pub fn open_range(&self) -> (f32, f32) {
		(self.left.min, self.right.min)
	}

	/// Whether the interval is well behaved.
	///
	/// If the left of the interval is well behaved, then the interval is well behaved.
	pub fn is_well_behaved(&self) -> bool {
		self.left.sign.is_well_behaved()
	}

	/// Whether the interval overlaps with another interval.
	pub fn overlaps_with(&self, other: &Self) -> bool {
		self.left.min < other.right.min && self.right.min > other.left.min
	}

	/// The interval of the intersection of the two intervals.
	pub fn range_intersection(&self, other: &Self) -> (f32, f32) {
		(self.left.min.max(other.left.min), self.right.min.min(other.right.min))
	}

	/// The full interval of the union of the two intervals.
	pub fn range_union(&self, other: &Self) -> (f32, f32) {
		(self.left.min.min(other.left.min), self.right.min.max(other.right.min))
	}

	/// The decomposition of the two intervals.
	///
	/// This gives a tuple:
	///     (1) who defines the sign boundary before the overlap
	///     (2) defines where the overlap begins (it ends at the after or whatever was the natural boundary before)
	///     (3) defines the sign boundary after the overlap
	///
	/// How the overlap is dealt with is specific to a combinator.
	pub fn decompose(
		&self,
		other: &Self,
	) -> (Option<SignBoundary>, Option<f32>, Option<SignBoundary>) {
		if !self.overlaps_with(other) {
			return (None, None, None);
		}

		// The interval before the overlap is given by whichever has the the lower left
		// If they have the same left bound then there is no before overlap.
		//
		// Observe that we are only dealing with the value of the left boundary here,
		// that's because the left boundary defines the value of an interval,
		// while the right merely gives the edge at which the value stops holding.
		let mut before_overlap = None;
		if self.left.min < other.left.min {
			before_overlap = Some(self.left.clone());
		} else if other.left.min < self.left.min {
			before_overlap = Some(other.left.clone());
		}

		// The interval after the overlap is given by whichever has the the higher right bound
		// If they have the same right bound then there is no after overlap.
		//
		// This has the effect of shifting returning to the lower bound on the right.
		//
		// If the righthand side/"other" sits across multiple lefthand side/"self" intervals,
		// then this ends up being overloaded in the next step.
		let mut after_overlap = None;
		if self.right.min > other.right.min {
			let mut pre_after_overlap = self.left.clone();
			pre_after_overlap.min = other.right.min;
			after_overlap = Some(pre_after_overlap);
		} else if other.right.min > self.right.min {
			let mut pre_after_overlap = other.left.clone();
			pre_after_overlap.min = self.right.min;
			after_overlap = Some(pre_after_overlap);
		}

		(before_overlap, Some(self.left.min.max(other.left.min)), after_overlap)
	}
}

#[derive(Debug, Clone, Default)]
pub struct SignUniformIntervals {
	boundaries: BTreeSet<SignBoundary>,
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
