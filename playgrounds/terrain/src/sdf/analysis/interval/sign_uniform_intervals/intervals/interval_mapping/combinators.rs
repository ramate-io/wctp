use super::IntervalMapping;
use crate::sdf::analysis::interval::PreSignUniformIntervals;

impl IntervalMapping {
	/// Computes the union of the interval mapping.
	pub fn union(self) -> PreSignUniformIntervals {
		let mut intervals = PreSignUniformIntervals::new();
		for (left_interval, right_intervals) in self.into_iter() {
			if let Some(left_interval) = left_interval {
				if right_intervals.is_empty() {
					intervals.insert_interval(left_interval);
				} else {
					for right_interval in right_intervals {
						let interval = left_interval.union(&right_interval);
						intervals.insert_boundary(interval);
					}
				}
			} else {
				for right_interval in right_intervals {
					intervals.insert_interval(right_interval);
				}
			}
		}
		intervals
	}

	/// Computes the difference of the interval mapping.
	pub fn difference(self) -> PreSignUniformIntervals {
		let mut intervals = PreSignUniformIntervals::new();
		for (left_interval, right_intervals) in self.into_iter() {
			if let Some(left_interval) = left_interval {
				if right_intervals.is_empty() {
					intervals.insert_interval(left_interval);
				} else {
					for right_interval in right_intervals {
						let interval = left_interval.difference(&right_interval);
						intervals.insert_boundary(interval);
					}
				}
			}
			// remaining right intervals are not intersecting and under difference operation
			// can be disregarded
		}
		intervals
	}
}

#[cfg(test)]
mod tests {
	use crate::sdf::analysis::interval::{PreSignUniformIntervals, Sign, SignBoundary};

	#[test]
	fn test_union() {
		let mut left_pre_intervals = PreSignUniformIntervals::new();
		left_pre_intervals.insert_boundary(SignBoundary { min: 0.0, sign: Sign::Negative });
		left_pre_intervals.insert_boundary(SignBoundary { min: 1.0, sign: Sign::Positive });
		left_pre_intervals.insert_boundary(SignBoundary { min: 2.0, sign: Sign::Negative });
		let left_intervals = left_pre_intervals.normalize();

		let mut right_pre_intervals = PreSignUniformIntervals::new();
		right_pre_intervals.insert_boundary(SignBoundary { min: 1.0, sign: Sign::Negative });
		right_pre_intervals.insert_boundary(SignBoundary { min: 2.0, sign: Sign::Positive });
		right_pre_intervals.insert_boundary(SignBoundary { min: 3.0, sign: Sign::Negative });
		let right_intervals = right_pre_intervals.normalize();

		let interval_mapping = left_intervals.interval_mapping(&right_intervals);
		let result = interval_mapping.union().normalize();

		let mut expected_intervals = PreSignUniformIntervals::new();
		expected_intervals.insert_boundary(SignBoundary { min: 0.0, sign: Sign::Negative });
		let expected_intervals = expected_intervals.normalize();

		assert_eq!(result, expected_intervals);
	}
}
