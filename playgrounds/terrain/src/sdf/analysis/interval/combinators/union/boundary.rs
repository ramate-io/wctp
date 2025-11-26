use crate::sdf::analysis::interval::{Sign, SignBoundary};

impl SignBoundary {
	/// Computes the pairwise union of boundaries.
	///
	/// On the whole, unions are just keeping all of the boundaries that are negative
	/// and only keeping positive when both boundaries are positive.
	pub fn union(&self, other: &SignBoundary) -> Vec<SignBoundary> {
		match other.sign {
			Sign::Negative => {
				// If other is negative, then we can just keep other.
				// We know this is negative from the other boundary.
				vec![other.clone()]
			}
			Sign::Positive => {
				// If both values are positive, this is realized here.
				//
				// If other is a lower boundary,
				// it would only be positive if a LHS boundary < n and intersecting with other were positive, w.l.o.g.
				if self.sign.is_positive() {
					vec![SignBoundary { min: self.min.max(other.min), sign: self.sign.clone() }]
				} else {
					vec![self.clone()]
				}
				// |++|-----------|++++|
				// a  b           c    d
				// |----|+++|---|++++|---|
				// 1    2   3   4    5   6
				// |--|-----|-----|++|---|
				// 1  b     3     c  5   6
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
		assert_eq!(result, vec![SignBoundary { min: 1.0, sign: Sign::Negative }]);
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
				SignBoundary { min: 1.0, sign: Sign::Negative },
				SignBoundary { min: 2.0, sign: Sign::Positive },
			]
		)
	}

	#[test]
	fn test_unions_over_rhs_many_complex() {
		let boundary = SignBoundary { min: 2.0, sign: Sign::Negative };
		let others = vec![
			SignBoundary { min: 1.0, sign: Sign::Negative },
			SignBoundary { min: 3.0, sign: Sign::Positive },
		];
		let result = boundary.unions_over(&others);
		assert_eq!(
			result,
			vec![
				// normalization will later combine these two
				SignBoundary { min: 1.0, sign: Sign::Negative },
				SignBoundary { min: 2.0, sign: Sign::Negative },
			]
		)
	}

	#[test]
	fn test_unions_on_simple() {
		let mapping = vec![
			(
				SignBoundary { min: 0.0, sign: Sign::Positive },
				vec![SignBoundary { min: 1.0, sign: Sign::Negative }],
			),
			(
				SignBoundary { min: 2.0, sign: Sign::Negative },
				vec![
					// because the RHS boundary from 1.0 intersects with both,
					// it should show up in both mappings.
					SignBoundary { min: 1.0, sign: Sign::Negative },
					SignBoundary { min: 3.0, sign: Sign::Positive },
				],
			),
		];
		let result = SignBoundary::unions_on(&mapping);
		assert_eq!(
			result,
			vec![
				// later normalization will combine...
				SignBoundary { min: 1.0, sign: Sign::Negative },
				SignBoundary { min: 1.0, sign: Sign::Negative },
				SignBoundary { min: 2.0, sign: Sign::Negative },
				// ...the three negative boundaries
			]
		)
	}

	#[test]
	fn test_unions_on_rhs_intersects_many() {
		let rhs = SignBoundary { min: -1.0, sign: Sign::Negative };

		let mapping = vec![
			(SignBoundary { min: 0.0, sign: Sign::Positive }, vec![rhs.clone()]),
			(SignBoundary { min: 2.0, sign: Sign::Negative }, vec![rhs.clone()]),
			(SignBoundary { min: 3.0, sign: Sign::Positive }, vec![rhs.clone()]),
			(SignBoundary { min: 4.0, sign: Sign::Negative }, vec![rhs.clone()]),
		];
		let result = SignBoundary::unions_on(&mapping);
		assert_eq!(
			result,
			vec![
				// this is unnormalized, but negative on the entire interval from -1.0
				SignBoundary { min: -1.0, sign: Sign::Negative },
				SignBoundary { min: -1.0, sign: Sign::Negative },
				SignBoundary { min: -1.0, sign: Sign::Negative },
				SignBoundary { min: -1.0, sign: Sign::Negative },
			]
		)
	}

	#[test]
	fn test_unions_on_rhs_intersects_many_then_flips() {
		let rhs_start = SignBoundary { min: -1.0, sign: Sign::Negative };
		let rhs_end = SignBoundary { min: 5.0, sign: Sign::Positive };
		let mapping = vec![
			(SignBoundary { min: 0.0, sign: Sign::Positive }, vec![rhs_start.clone()]),
			(SignBoundary { min: 2.0, sign: Sign::Negative }, vec![rhs_start.clone()]),
			(SignBoundary { min: 3.0, sign: Sign::Positive }, vec![rhs_start.clone()]),
			(
				SignBoundary { min: 4.0, sign: Sign::Negative },
				vec![rhs_start.clone(), rhs_end.clone()],
			),
			(SignBoundary { min: 5.0, sign: Sign::Positive }, vec![rhs_end.clone()]),
		];
		let result = SignBoundary::unions_on(&mapping);
		assert_eq!(
			result,
			vec![
				SignBoundary { min: -1.0, sign: Sign::Negative },
				SignBoundary { min: -1.0, sign: Sign::Negative },
				SignBoundary { min: -1.0, sign: Sign::Negative },
				SignBoundary { min: -1.0, sign: Sign::Negative },
				SignBoundary { min: 4.0, sign: Sign::Negative },
				SignBoundary { min: 5.0, sign: Sign::Positive },
			]
		)
	}
}
