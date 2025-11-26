use crate::sdf::analysis::interval::{Sign, SignBoundary};

impl SignBoundary {
	pub fn union(&self, other: &SignBoundary) -> Vec<SignBoundary> {
		match other.sign {
			Sign::Negative => {
				if other.min < self.min {
					// this negates self entirely as far as this pair is concerned
					// this should have been included before this point,
					// so the vector would be empty.
					// but this is easier to fix in a normalization pass
					// so, we just keep the lower boundary
					vec![other.clone()]
				} else {
					vec![self.clone(), other.clone()]
				}
			}
			Sign::Positive => {
				// whatever the sign on self is at the max of it and other
				vec![SignBoundary { min: self.min.max(other.min), sign: self.sign.clone() }]
			}
			Sign::Top => {
				// This is unknown from the lowest point
				vec![SignBoundary { min: self.min.min(other.min), sign: Sign::Top }]
			}
			Sign::Bottom => {
				// This is undefined from the lowest point
				vec![SignBoundary { min: self.min.min(other.min), sign: Sign::Bottom }]
			}
		}
	}

	/// Computes the union of this boundary with a list of other boundaries.
	///
	/// The assumption here is that the other boundaries are all of the intersecting RHS boundaries
	/// before the next LHS boundary.
	pub fn unions_over(&self, others_before_next: &Vec<SignBoundary>) -> Vec<SignBoundary> {
		others_before_next
			.into_iter()
			.map(|other| self.union(&other))
			.flatten()
			.collect()
	}

	pub fn unions_on(mapping: &Vec<(SignBoundary, Vec<SignBoundary>)>) -> Vec<SignBoundary> {
		mapping
			.into_iter()
			.map(|(boundary, others)| boundary.unions_over(&others))
			.flatten()
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_unions_over_rhs_one_negative() {
		let boundary = SignBoundary { min: 0.0, sign: Sign::Positive };
		let others = vec![SignBoundary { min: 1.0, sign: Sign::Negative }];
		let result = boundary.unions_over(&others);
		assert_eq!(
			result,
			vec![
				SignBoundary { min: 0.0, sign: Sign::Positive },
				SignBoundary { min: 1.0, sign: Sign::Negative }
			]
		);
	}

	#[test]
	fn test_unions_over_rhs_before_lhs_positive() {
		let boundary = SignBoundary { min: 0.0, sign: Sign::Negative };
		let others = vec![SignBoundary { min: -1.0, sign: Sign::Positive }];
		let result = boundary.unions_over(&others);
		assert_eq!(result, vec![SignBoundary { min: 0.0, sign: Sign::Negative }]);
	}

	#[test]
	fn test_unions_over_rhs_before_lhs_negative() {
		let boundary = SignBoundary { min: 0.0, sign: Sign::Positive };
		let others = vec![SignBoundary { min: -1.0, sign: Sign::Negative }];
		let result = boundary.unions_over(&others);
		assert_eq!(result, vec![SignBoundary { min: -1.0, sign: Sign::Negative }]);
	}

	#[test]
	fn test_unions_over_rhs_many_simple() {
		let boundary = SignBoundary { min: 0.0, sign: Sign::Positive };
		let others = vec![
			SignBoundary { min: 1.0, sign: Sign::Negative },
			SignBoundary { min: 2.0, sign: Sign::Positive },
		];
		let result = boundary.unions_over(&others);
		assert_eq!(
			result,
			vec![
				SignBoundary { min: 0.0, sign: Sign::Positive },
				SignBoundary { min: 1.0, sign: Sign::Negative },
				SignBoundary { min: 2.0, sign: Sign::Positive },
			]
		)
	}
}
