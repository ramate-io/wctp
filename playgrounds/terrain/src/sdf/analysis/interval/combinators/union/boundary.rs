use crate::sdf::analysis::interval::{Sign, SignBoundary};

impl SignBoundary {
	pub fn union(&self, other: &SignBoundary) -> Vec<SignBoundary> {
		match other.sign {
			Sign::Negative => {
				if other.min < self.min {
					// this negates self entirely as far as this pair is concerned
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
}
