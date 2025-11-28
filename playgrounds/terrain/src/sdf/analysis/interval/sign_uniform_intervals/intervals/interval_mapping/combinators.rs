use super::IntervalMapping;
use crate::sdf::analysis::interval::PreSignUniformIntervals;

impl IntervalMapping {
	pub fn union(self) -> PreSignUniformIntervals {
		let mut intervals = PreSignUniformIntervals::new();
		for (left_interval, right_intervals) in self.into_iter() {
			if let Some(left_interval) = left_interval {
				for right_interval in right_intervals {
					let interval = left_interval.union(&right_interval);
					intervals.insert_boundary(interval);
				}
			} else {
				for right_interval in right_intervals {
					intervals.insert_interval(right_interval);
				}
			}
		}
		intervals
	}
}
