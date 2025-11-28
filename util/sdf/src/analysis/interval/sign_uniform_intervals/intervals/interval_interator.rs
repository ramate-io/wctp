use crate::analysis::interval::{SignBoundary, SignUniformInterval};

/// Iterator that consumes the SignUniformIntervals structure.
/// Iterates left, right pairs beginning with the top constant, then through the members of the set, then ending with the bottom constant.
pub struct SignUniformIntervalsIterator {
	intervals: Vec<SignBoundary>,
	index: usize,
	emitted_top: bool,
}

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

impl SignUniformIntervalsIterator {
	pub fn new(intervals: Vec<SignBoundary>) -> Self {
		Self { intervals, index: 0, emitted_top: false }
	}
}
