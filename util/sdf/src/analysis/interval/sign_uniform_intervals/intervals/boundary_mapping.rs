pub mod combinators;

use crate::analysis::interval::{SignBoundary, SignUniformIntervals};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct BoundaryMapping {
	mapping: BTreeMap<Option<SignBoundary>, BTreeSet<SignBoundary>>,
}

impl BoundaryMapping {
	pub fn into_iter(self) -> impl Iterator<Item = (Option<SignBoundary>, BTreeSet<SignBoundary>)> {
		self.mapping.into_iter()
	}
}

impl SignUniformIntervals {
	/// Maps each boundary to all the boundaries which intersect with it given the known LHS intervals.
	pub fn boundary_mapping(&self, other: &Self) -> BoundaryMapping {
		let mut boundary_mapping = BTreeMap::new();
		let interval_mapping = self.interval_mapping(other);

		for (interval, other_intervals) in interval_mapping.into_iter() {
			// The LHS boundary is given to us by the left of the interval.
			let left = interval.map(|interval| interval.left);

			// The RHS boundaries are give to use by the lefts of each of the other intervals.
			for other_interval in other_intervals {
				boundary_mapping
					.entry(left.clone())
					.or_insert(BTreeSet::new())
					.insert(other_interval.left);
			}
		}

		BoundaryMapping { mapping: boundary_mapping }
	}
}
