use crate::sdf::analysis::interval::{Sign, SignBoundary};

impl SignBoundary {
	pub fn difference(&self, other: &SignBoundary) -> Vec<SignBoundary> {
		match other.sign {
			Sign::Negative => {
				if other.min < self.min {
					// this negates self entirely as far as this pair is concerned
					vec![other.flip()]
				} else {
					vec![self.clone(), other.flip()]
				}
			}
			Sign::Positive => {
				// You can just keep self.
				// If self is positive, then that positivity is preserved.
				// If self is negative, then that negativity is preserved--valid union.
				// If self is top, then topness is preserved. The previous interval would can
				// take whatever the value up to point.
				// Bottom is the same logic as top.
				vec![self.clone()]
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
		assert_eq!(
			result,
			vec![
				SignBoundary { min: 0.0, sign: Sign::Negative },
				SignBoundary { min: 1.0, sign: Sign::Positive }
			]
		);
	}

	#[test]
	fn test_difference_over_rhs_before_lhs_negative() {
		let boundary = SignBoundary { min: 0.0, sign: Sign::Negative };
		let others = vec![SignBoundary { min: -1.0, sign: Sign::Negative }];
		let result = boundary.differences_over(&others);
		assert_eq!(result, vec![SignBoundary { min: -1.0, sign: Sign::Positive }]);
	}

	#[test]
	fn test_unions_over_rhs_before_lhs_positive() {
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
				SignBoundary { min: 0.0, sign: Sign::Negative },
				SignBoundary { min: 0.0, sign: Sign::Negative },
				SignBoundary { min: 2.0, sign: Sign::Positive },
			]
		)
	}
}
