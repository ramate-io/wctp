pub mod combinators;

use crate::analysis::interval::{Sign, SignBoundary};

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

	/// The interval of the intersection of the two intervals.
	pub fn range_intersection(&self, other: &Self) -> (f32, f32) {
		(self.left.min.max(other.left.min), self.right.min.min(other.right.min))
	}

	/// Whether the interval intersects with another interval.
	pub fn intersects_with(&self, other: &Self) -> bool {
		let (min, max) = self.range_intersection(other);
		min < max
	}

	/// The full interval of the union of the two intervals.
	pub fn range_union(&self, other: &Self) -> (f32, f32) {
		(self.left.min.min(other.left.min), self.right.min.max(other.right.min))
	}

	/// Gets the rightmost boundary between two intervals.
	pub fn leftmost_right_boundary(&self, other: &Self) -> SignBoundary {
		if self.right.min > other.right.min {
			other.right.clone()
		} else if other.right.min > self.right.min {
			self.right.clone()
		} else {
			// The bounds conincide,
			// we use the leftmost behavior,
			// and let the next boundary decide.
			//
			// For all set iterations, SignBoundary::bottom() would already exist,
			// hence this is safe.
			//
			// Bottom is better than Top because Top should be to the left of all values.
			// While in the set representation that this is here used as the rightmost boundary,
			// for semantic reasons, it makes more sense to use Bottom.
			SignBoundary::bottom()
		}
	}

	/// Gets the undecided interval between two intervals.
	pub fn undecided_interval(&self, other: &Self) -> UndecidedBoundary {
		let (min, _right) = self.range_intersection(other);
		let left_sign = self.left.sign.clone();
		let right_sign = other.left.sign.clone();
		UndecidedBoundary { min, left_sign, right_sign }
	}
}

#[derive(Debug, Clone)]
pub struct UndecidedBoundary {
	pub min: f32,
	pub left_sign: Sign,
	pub right_sign: Sign,
}

impl UndecidedBoundary {
	/// Computes the union of the
	pub fn union(&self) -> SignBoundary {
		let sign_union = self.left_sign.union(&self.right_sign);
		SignBoundary { min: self.min, sign: sign_union }
	}

	/// Computes the difference of the undecided interval.
	pub fn difference(&self) -> SignBoundary {
		let sign_difference = self.left_sign.difference(&self.right_sign);
		SignBoundary { min: self.min, sign: sign_difference }
	}
}
