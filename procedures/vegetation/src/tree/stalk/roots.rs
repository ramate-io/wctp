use sdf::{Sdf, CapsuleSdf};
use bevy::prelude::*;

/// A root system SDF - composed of multiple root segments
/// Roots typically branch downward and outward from the trunk base
pub struct RootSdf {
	/// Base point where roots originate (typically trunk base)
	pub origin: Vec3,
	/// Root segments: (start, end, radius)
	pub segments: Vec<(Vec3, Vec3, f32)>,
}

impl RootSdf {
	pub fn new(origin: Vec3) -> Self {
		Self { origin, segments: Vec::new() }
	}

	/// Add a root segment
	pub fn add_segment(&mut self, start: Vec3, end: Vec3, radius: f32) {
		self.segments.push((start, end, radius));
	}
}

impl Sdf for RootSdf {
	fn distance(&self, p: Vec3) -> f32 {
		if self.segments.is_empty() {
			// No roots - return large positive distance
			return f32::MAX;
		}

		// Union all root segments
		// For simplicity, we'll compute the minimum distance across all segments
		// A more sophisticated implementation could use a proper union structure
		let mut min_dist = f32::MAX;
		
		for (start, end, radius) in &self.segments {
			let capsule = CapsuleSdf::new(*start, *end, *radius);
			let dist = capsule.distance(p);
			min_dist = min_dist.min(dist);
		}
		
		min_dist
	}
}

