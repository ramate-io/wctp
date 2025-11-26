use crate::sdf::analysis::interval::{Sign, SignBoundary};

impl SignBoundary {
	pub fn intersection(&self, other: &SignBoundary) -> Vec<SignBoundary> {
		match (&self.sign, &other.sign) {
			(Sign::Top, Sign::Top) => {
				// Top gets normalized
				vec![SignBoundary { min: self.min.min(other.min), sign: Sign::Top }]
			}
			(Sign::Top, _) => {
				// Top gets preserved
				vec![self.clone()]
			}
			(_, Sign::Top) => {
				// Top gets preserved
				vec![other.clone()]
			}
			(Sign::Bottom, Sign::Bottom) => {
				// Bottom gets normalized
				vec![SignBoundary { min: self.min.min(other.min), sign: Sign::Bottom }]
			}
			(Sign::Bottom, _) => {
				// Bottom gets preserved
				vec![other.clone()]
			}
			(_, Sign::Bottom) => {
				// Bottom gets preserved
				vec![self.clone()]
			}
			(Sign::Negative, Sign::Negative) => {
				// If both are negative, the result is negative at the max of the two
				vec![SignBoundary { min: self.min.max(other.min), sign: Sign::Negative }]
			}
			(Sign::Positive, Sign::Positive) => {
				// If both are positive, we normalized to the min of the two.
				vec![SignBoundary { min: self.min.min(other.min), sign: Sign::Positive }]
			}
			(Sign::Negative, Sign::Positive) => {
				// If other is positive, we know we are positive from where other is positive.
				vec![SignBoundary { min: other.min, sign: Sign::Positive }]
			}
			(Sign::Positive, Sign::Negative) => {
				// If self is positive, we know we are positive from where self is positive.
				vec![SignBoundary { min: self.min, sign: Sign::Positive }]
			}
		}
	}
}
