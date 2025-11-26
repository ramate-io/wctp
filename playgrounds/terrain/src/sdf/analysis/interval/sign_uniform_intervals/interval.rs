use crate::sdf::analysis::interval::SignBoundary;

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
