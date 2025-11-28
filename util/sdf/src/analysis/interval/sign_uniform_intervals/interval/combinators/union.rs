use crate::analysis::interval::{SignBoundary, SignUniformInterval};

impl SignUniformInterval {
	pub fn union(&self, other: &Self) -> SignBoundary {
		self.undecided_interval(other).union()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::analysis::interval::Sign;
	use crate::analysis::interval::SignBoundary;

	#[test]
	fn test_lower_left_negative_union() {
		let interval1 = SignUniformInterval {
			left: SignBoundary { min: 0.0, sign: Sign::Negative },
			right: SignBoundary { min: 2.0, sign: Sign::Positive },
		};
		let interval2 = SignUniformInterval {
			left: SignBoundary { min: 1.0, sign: Sign::Top },
			right: SignBoundary { min: 2.0, sign: Sign::Positive },
		};
		let result = interval1.union(&interval2);
		assert_eq!(result, SignBoundary { min: 1.0, sign: Sign::Negative });
	}

	#[test]
	fn test_lower_left_positive_union() {
		let interval1 = SignUniformInterval {
			left: SignBoundary { min: 0.0, sign: Sign::Positive },
			right: SignBoundary { min: 2.0, sign: Sign::Negative },
		};
		let interval2 = SignUniformInterval {
			left: SignBoundary { min: 1.0, sign: Sign::Top },
			right: SignBoundary { min: 2.0, sign: Sign::Positive },
		};

		let result = interval1.union(&interval2);
		assert_eq!(result, SignBoundary { min: 1.0, sign: Sign::Top });
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
		let result = interval1.union(&interval2);
		assert_eq!(result, SignBoundary { min: 2.0, sign: Sign::Negative });
	}

	#[test]
	fn test_contain_adds_to() {
		let interval1 = SignUniformInterval {
			left: SignBoundary { min: 0.0, sign: Sign::Positive },
			right: SignBoundary { min: 3.0, sign: Sign::Negative },
		};
		let interval2 = SignUniformInterval {
			left: SignBoundary { min: 1.0, sign: Sign::Negative },
			right: SignBoundary { min: 2.0, sign: Sign::Positive },
		};

		let result = interval1.union(&interval2);
		assert_eq!(result, SignBoundary { min: 1.0, sign: Sign::Negative });
	}
}
