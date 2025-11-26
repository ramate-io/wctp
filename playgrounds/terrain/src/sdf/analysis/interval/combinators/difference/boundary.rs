use crate::sdf::analysis::interval::{Sign, SignBoundary};

impl SignBoundary {
	pub fn difference(&self, other: &SignBoundary) -> Vec<SignBoundary> {
		match other.sign {
			Sign::Negative => {
				// this always just clears out from where the other is negative.
				vec![other.flip()]
				// Note: you could argue a normalized boundary mapping
				// should never have two RHS negative boundaries back to back.
				// In this case, you could add the self.clone() boundary here.
				// However, we can generally assume that the previous mapping entry
				// would preserve self.clone() if it was not RHS negative.
				// Hence, we don't have to make as strong of an assumption
				// for only returning other.flip() to be correct.
			}
			Sign::Positive => {
				// This is just self from wherever the other is positive.
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

	pub fn differences_over(&self, others_before_next: &Vec<SignBoundary>) -> Vec<SignBoundary> {
		others_before_next
			.into_iter()
			.map(|other| self.difference(&other))
			.flatten()
			.collect()
	}

	pub fn differences_on(mapping: &Vec<(SignBoundary, Vec<SignBoundary>)>) -> Vec<SignBoundary> {
		mapping
			.into_iter()
			.map(|(boundary, others)| boundary.differences_over(&others))
			.flatten()
			.collect()
	}
}

#[cfg(test)]
pub mod test {

	use super::*;

	#[test]
	fn test_difference_over_rhs_one_negative() {
		let boundary = SignBoundary { min: 0.0, sign: Sign::Negative };
		let others = vec![SignBoundary { min: 1.0, sign: Sign::Negative }];
		let result = boundary.differences_over(&others);
		assert_eq!(result, vec![SignBoundary { min: 1.0, sign: Sign::Positive }]);
	}

	#[test]
	fn test_difference_over_rhs_before_lhs_negative() {
		let boundary = SignBoundary { min: 0.0, sign: Sign::Negative };
		let others = vec![SignBoundary { min: -1.0, sign: Sign::Negative }];
		let result = boundary.differences_over(&others);
		assert_eq!(result, vec![SignBoundary { min: -1.0, sign: Sign::Positive }]);
	}

	#[test]
	fn test_differences_over_rhs_before_lhs_positive() {
		let boundary = SignBoundary { min: 0.0, sign: Sign::Negative };
		let others = vec![SignBoundary { min: -1.0, sign: Sign::Positive }];
		let result = boundary.differences_over(&others);
		assert_eq!(result, vec![SignBoundary { min: 0.0, sign: Sign::Negative }]);
	}

	#[test]
	fn test_differences_over_rhs_many_simple() {
		let boundary = SignBoundary { min: 0.0, sign: Sign::Negative };
		let others = vec![
			SignBoundary { min: 1.0, sign: Sign::Positive },
			SignBoundary { min: 2.0, sign: Sign::Negative },
		];
		let result = boundary.differences_over(&others);
		assert_eq!(
			result,
			vec![
				SignBoundary { min: 1.0, sign: Sign::Negative },
				SignBoundary { min: 1.0, sign: Sign::Negative },
				SignBoundary { min: 2.0, sign: Sign::Positive },
			]
		)
	}

	#[test]
	fn test_differences_over_rhs_many_complex() {
		let boundary = SignBoundary { min: 2.0, sign: Sign::Negative };
		let others = vec![
			SignBoundary { min: 1.0, sign: Sign::Negative },
			SignBoundary { min: 3.0, sign: Sign::Positive },
		];
		let result = boundary.differences_over(&others);
		assert_eq!(
			result,
			vec![
				// normalization will later combine these two
				SignBoundary { min: 1.0, sign: Sign::Positive },
				SignBoundary { min: 3.0, sign: Sign::Negative },
			]
		)
	}
}
