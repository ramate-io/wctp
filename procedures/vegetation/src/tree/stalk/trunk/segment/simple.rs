use super::{
	join_point::{JoinPoint, JoinType},
	SegmentConfig,
};
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use sdf::Sdf;

/// Simple trunk segment: noisy cylinder with trunk join points on top and bottom
pub struct SimpleTrunkSegment {
	config: SegmentConfig,
	noise: Perlin,
}

impl SimpleTrunkSegment {
	pub fn new(config: SegmentConfig) -> Self {
		let noise = Perlin::new(config.seed);
		Self { config, noise }
	}

	/// Get join points: one at bottom (0.0) and one at top (1.0)
	pub fn join_points(&self) -> Vec<JoinPoint> {
		vec![
			JoinPoint {
				unit_position: 0.0,
				angle: 0.0, // Angle doesn't matter for single trunk join
				join_type: JoinType::Trunk,
			},
			JoinPoint { unit_position: 1.0, angle: 0.0, join_type: JoinType::Trunk },
		]
	}
}

impl Sdf for SimpleTrunkSegment {
	/// TODO: there is a bug that gives this some slightly weird sharp facets.
	/// For now, we're going to keep moving because it's a small aesthetic issue, but it should be fixed at some point.
	fn distance(&self, p: Vec3) -> f32 {
		// Clamp y to [0, 1] for the segment
		let y = p.y;
		let normalized_y = y.clamp(0.0, 1.0);

		// Interpolate radius along the segment
		let radius =
			self.config.base_radius * (1.0 - normalized_y) + self.config.top_radius * normalized_y;

		// Distance from center in XZ plane
		let xz_dist = (p.x * p.x + p.z * p.z).sqrt();

		// Base cylinder distance
		let mut dist = xz_dist - radius;

		// Add noise perturbation for surface variation
		let noise_value = self.noise.get([
			p.x as f64 * self.config.noise_frequency as f64,
			y as f64 * self.config.noise_frequency as f64,
			p.z as f64 * self.config.noise_frequency as f64,
		]) as f32;
		dist += noise_value * self.config.noise_amplitude;

		// Handle end caps
		if y < 0.0 {
			// Below bottom - distance to bottom cap
			let cap_dist = -y;
			dist = dist.max(cap_dist);
		} else if y > 1.0 {
			// Above top - distance to top cap
			let cap_dist = y - 1.0;
			dist = dist.max(cap_dist);
		}

		dist
	}
}
