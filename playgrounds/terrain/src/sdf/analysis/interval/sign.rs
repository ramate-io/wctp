#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sign {
	/// The sign is unknown.
	Top,
	/// The sign is negative.
	Negative,
	/// The sign is positive.
	Positive,
	/// The sign is known but undefined.
	Bottom,
}

impl Sign {
	/// Returns true if the sign is well behaved.
	pub fn is_well_behaved(&self) -> bool {
		matches!(self, Sign::Negative | Sign::Positive)
	}

	/// Returns true if sign is negative
	pub fn is_negative(&self) -> bool {
		matches!(self, Sign::Negative)
	}

	/// Returns true if sign is negative
	pub fn is_positive(&self) -> bool {
		matches!(self, Sign::Positive)
	}

	/// Returns the union of the two signs.
	pub fn union(&self, other: &Self) -> Self {
		match (self, other) {
			(Sign::Negative, _) => Sign::Negative,
			(_, Sign::Negative) => Sign::Negative,
			(Sign::Positive, Sign::Positive) => Sign::Positive,
			_ => Sign::Top,
		}
	}

	/// Flips the sign if negative, otherwise returns the sign.
	pub fn flip(&self) -> Self {
		if self == &Sign::Negative {
			Sign::Positive
		} else {
			self.clone()
		}
	}

	/// Returns the difference of the two signs.
	pub fn difference(&self, other: &Self) -> Self {
		match (self, other) {
			// whatever the self sign is, if the other is negative, then the result is positive
			(_, Sign::Negative) => Sign::Positive,
			// otherwise, the sign stays the same
			_ => self.clone(),
		}
	}
}
