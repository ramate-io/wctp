use crate::sdf::analysis::interval::Sign;

/// The sign is uniform from the min to some next boundary which will be placed in the intervals.
#[derive(Debug, Clone)]
pub struct SignBoundary {
	pub min: f32,
	pub sign: Sign,
}

impl SignBoundary {
	/// The sign is uniformly unknown from negative infinity.
	pub const fn top() -> Self {
		Self { min: f32::NEG_INFINITY, sign: Sign::Top }
	}

	/// The sign is uniformly undefined from positive infinity.
	pub const fn bottom() -> Self {
		Self { min: f32::INFINITY, sign: Sign::Bottom }
	}

	/// Constructrs a version of the boundary with the sign flipped.
	pub fn flip(&self) -> Self {
		Self { min: self.min, sign: self.sign.flip() }
	}
}

impl PartialOrd for SignBoundary {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// compare min then sign
		Some(
			self.min
				.partial_cmp(&other.min)
				.unwrap_or(std::cmp::Ordering::Equal)
				.then_with(|| {
					self.sign.partial_cmp(&other.sign).unwrap_or(std::cmp::Ordering::Equal)
				}),
		)
	}
}

impl PartialEq for SignBoundary {
	fn eq(&self, other: &Self) -> bool {
		self.min == other.min && self.sign == other.sign
	}
}

impl Eq for SignBoundary {}

impl Ord for SignBoundary {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.partial_cmp(other).unwrap()
	}
}
