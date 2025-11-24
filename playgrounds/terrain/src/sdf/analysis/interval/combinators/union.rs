use super::super::{SignBoundary, SignUniformInterval};

impl SignUniformInterval {
	pub fn union(&self, other: &Self) -> Vec<SignBoundary> {
		let mut result = Vec::new();

		// Decompose the intervals
		let (before_overlap, overlap, after_overlap) = self.decompose(other);

		if let Some(start_undecided) = overlap {
			// Whatever comes before the interval truncate it to the interval min.
			if let Some(before_overlap) = before_overlap {
				result.push(before_overlap);
			}

			// Whatever comes after the interval truncate it to the interval max.
			if let Some(after_overlap) = after_overlap {
				result.push(after_overlap);
			}

			result.push(SignBoundary {
				min: start_undecided,
				sign: self.left.sign.union(&other.left.sign),
			});

			result
		} else {
			// If there is not overlap, return all of the original boundaries.
			result.push(self.left.clone());
			result.push(self.right.clone());
			result.push(other.left.clone());
			result.push(other.right.clone());
			result
		}
	}
}
