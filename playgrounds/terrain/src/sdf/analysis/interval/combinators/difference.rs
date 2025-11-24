use super::super::{SignBoundary, SignUniformInterval};

impl SignUniformInterval {
	pub fn difference(&self, other: &Self) -> Vec<SignBoundary> {
		let mut result = Vec::new();

		// Decompose the intervals
		let (before_overlap, overlap, after_overlap) = self.decompose(other);

		if let Some(start_undecided) = overlap {
			// Whatever comes before the intersection.
			if let Some(before_overlap) = &before_overlap {
				result.push(before_overlap.clone());
			}

			// if the sign union is equivalent to the lower bound, just use the lower bound
			// this is a normalization measure
			let sign_difference = self.left.sign.difference(&other.left.sign);
			match before_overlap {
				Some(before_overlap) => {
					if sign_difference != before_overlap.sign {
						result.push(SignBoundary { min: start_undecided, sign: sign_difference });
					}
				}
				None => {
					result.push(SignBoundary { min: start_undecided, sign: sign_difference });
				}
			}

			// Whatever comes after the intersection.
			if let Some(after_overlap) = after_overlap {
				result.push(after_overlap);
			}

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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sdf::analysis::interval::Sign;

	#[test]
	fn test_lower_left_negative_difference() {
		let interval1 = SignUniformInterval {
			left: SignBoundary { min: 0.0, sign: Sign::Negative },
			right: SignBoundary { min: 2.0, sign: Sign::Positive },
		};
		let interval2 = SignUniformInterval {
			left: SignBoundary { min: 1.0, sign: Sign::Negative },
			right: SignBoundary { min: 2.0, sign: Sign::Positive },
		};
		let result = interval1.difference(&interval2);
		assert_eq!(
			result,
			vec![
				SignBoundary { min: 0.0, sign: Sign::Negative },
				SignBoundary { min: 1.0, sign: Sign::Positive },
			]
		);
	}

	#[test]
	fn test_lower_left_positive_difference() {
		let interval1 = SignUniformInterval {
			left: SignBoundary { min: 0.0, sign: Sign::Positive },
			right: SignBoundary { min: 2.0, sign: Sign::Negative },
		};
		let interval2 = SignUniformInterval {
			left: SignBoundary { min: 1.0, sign: Sign::Negative },
			right: SignBoundary { min: 2.0, sign: Sign::Positive },
		};
		let result = interval1.difference(&interval2);
		assert_eq!(result, vec![SignBoundary { min: 0.0, sign: Sign::Positive },]);
	}

	#[test]
	fn test_left_match() {
		let interval1 = SignUniformInterval {
			left: SignBoundary { min: 2.0, sign: Sign::Negative },
			right: SignBoundary { min: 3.0, sign: Sign::Positive },
		};
		let interval2 = SignUniformInterval {
			left: SignBoundary { min: 2.0, sign: Sign::Positive },
			right: SignBoundary { min: 4.0, sign: Sign::Top },
		};
		let result = interval1.difference(&interval2);
		assert_eq!(
			result,
			vec![
				SignBoundary { min: 2.0, sign: Sign::Negative },
				SignBoundary { min: 3.0, sign: Sign::Positive },
			]
		);
	}

	#[test]
	fn test_contain_carves_out() {
		let interval1 = SignUniformInterval {
			left: SignBoundary { min: 0.0, sign: Sign::Negative },
			right: SignBoundary { min: 3.0, sign: Sign::Positive },
		};
		let interval2 = SignUniformInterval {
			left: SignBoundary { min: 1.0, sign: Sign::Negative },
			right: SignBoundary { min: 2.0, sign: Sign::Positive },
		};

		let result = interval1.difference(&interval2);
		assert_eq!(
			result,
			vec![
				SignBoundary { min: 0.0, sign: Sign::Negative },
				SignBoundary { min: 1.0, sign: Sign::Positive },
				SignBoundary { min: 2.0, sign: Sign::Positive },
			]
		);
	}
}
