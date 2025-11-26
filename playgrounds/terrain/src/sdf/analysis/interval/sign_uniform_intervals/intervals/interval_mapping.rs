use crate::sdf::analysis::interval::{SignUniformInterval, SignUniformIntervals};
use std::collections::{BTreeMap, BTreeSet};

impl SignUniformIntervals {
	/// Computes the overlaps of the left interval set with the right interval set.
	pub fn interval_mapping(
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
