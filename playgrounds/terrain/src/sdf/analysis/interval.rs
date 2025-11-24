use std::collections::BTreeSet;

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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_empty_intervals() {
		let intervals = SignUniformIntervals::default();
		let pairs: Vec<_> = intervals.into_iter().collect();

		assert_eq!(pairs.len(), 1);
		assert_eq!(pairs[0].0.min, f32::NEG_INFINITY);
		assert_eq!(pairs[0].0.sign, Sign::Top);
		assert_eq!(pairs[0].1.min, f32::INFINITY);
		assert_eq!(pairs[0].1.sign, Sign::Bottom);
	}

	#[test]
	fn test_single_interval() {
		let mut intervals = SignUniformIntervals::default();
		intervals.intervals.insert(SignUniform { min: 0.0, sign: Sign::Negative });

		let pairs: Vec<_> = intervals.into_iter().collect();

		assert_eq!(pairs.len(), 2);

		// First pair: (top, first_element)
		assert_eq!(pairs[0].0.min, f32::NEG_INFINITY);
		assert_eq!(pairs[0].0.sign, Sign::Top);
		assert_eq!(pairs[0].1.min, 0.0);
		assert_eq!(pairs[0].1.sign, Sign::Negative);

		// Second pair: (first_element, bottom)
		assert_eq!(pairs[1].0.min, 0.0);
		assert_eq!(pairs[1].0.sign, Sign::Negative);
		assert_eq!(pairs[1].1.min, f32::INFINITY);
		assert_eq!(pairs[1].1.sign, Sign::Bottom);
	}

	#[test]
	fn test_multiple_intervals() {
		let mut intervals = SignUniformIntervals::default();
		intervals.intervals.insert(SignUniform { min: 0.0, sign: Sign::Negative });
		intervals.intervals.insert(SignUniform { min: 5.0, sign: Sign::Positive });
		intervals.intervals.insert(SignUniform { min: 10.0, sign: Sign::Negative });

		let pairs: Vec<_> = intervals.into_iter().collect();

		assert_eq!(pairs.len(), 4);

		// First pair: (top, first_element)
		assert_eq!(pairs[0].0.min, f32::NEG_INFINITY);
		assert_eq!(pairs[0].0.sign, Sign::Top);
		assert_eq!(pairs[0].1.min, 0.0);
		assert_eq!(pairs[0].1.sign, Sign::Negative);

		// Second pair: (0.0, 5.0)
		assert_eq!(pairs[1].0.min, 0.0);
		assert_eq!(pairs[1].0.sign, Sign::Negative);
		assert_eq!(pairs[1].1.min, 5.0);
		assert_eq!(pairs[1].1.sign, Sign::Positive);

		// Third pair: (5.0, 10.0)
		assert_eq!(pairs[2].0.min, 5.0);
		assert_eq!(pairs[2].0.sign, Sign::Positive);
		assert_eq!(pairs[2].1.min, 10.0);
		assert_eq!(pairs[2].1.sign, Sign::Negative);

		// Last pair: (last_element, bottom)
		assert_eq!(pairs[3].0.min, 10.0);
		assert_eq!(pairs[3].0.sign, Sign::Negative);
		assert_eq!(pairs[3].1.min, f32::INFINITY);
		assert_eq!(pairs[3].1.sign, Sign::Bottom);
	}

	#[test]
	fn test_interval_ordering() {
		let mut intervals = SignUniformIntervals::default();
		intervals.intervals.insert(SignUniform { min: 10.0, sign: Sign::Positive });
		intervals.intervals.insert(SignUniform { min: 0.0, sign: Sign::Negative });
		intervals.intervals.insert(SignUniform { min: 5.0, sign: Sign::Positive });

		let pairs: Vec<_> = intervals.into_iter().collect();

		// Should be ordered by min value regardless of insertion order
		assert_eq!(pairs.len(), 4);
		assert_eq!(pairs[0].1.min, 0.0);
		assert_eq!(pairs[1].0.min, 0.0);
		assert_eq!(pairs[1].1.min, 5.0);
		assert_eq!(pairs[2].0.min, 5.0);
		assert_eq!(pairs[2].1.min, 10.0);
		assert_eq!(pairs[3].0.min, 10.0);
	}

	#[test]
	fn test_iterator_consumes() {
		let mut intervals = SignUniformIntervals::default();
		intervals.intervals.insert(SignUniform { min: 0.0, sign: Sign::Negative });

		let mut iter = intervals.into_iter();
		assert!(iter.next().is_some());
		assert!(iter.next().is_some());
		assert!(iter.next().is_none());
	}
}
