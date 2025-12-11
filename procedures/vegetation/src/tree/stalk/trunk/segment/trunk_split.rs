use super::{
	join_point::{JoinPoint, JoinPointSdf, JoinType},
	simple::SimpleTrunkSegment,
	SegmentConfig,
};
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use sdf::Sdf;

/// Trunk split segment: trunk segment with n trunk join points on the top disc
/// The trunk draws into these join points (tapers toward them)
/// Union of simple trunk segment with join point bumps placed slightly below the top
pub struct TrunkSplitSegment {
	base_segment: SimpleTrunkSegment,
	num_splits: usize,
	config: SegmentConfig,
	split_seed_offset: u32,
}

impl TrunkSplitSegment {
	pub fn new(config: SegmentConfig, num_splits: usize) -> Self {
		Self {
			base_segment: SimpleTrunkSegment::new(config.clone()),
			num_splits,
			config,
			split_seed_offset: 2000, // Offset for split placement noise
		}
	}

	/// Get join points: trunk join at bottom, n trunk joins at top
	pub fn join_points(&self) -> Vec<JoinPoint> {
		let mut points =
			vec![JoinPoint { unit_position: 0.0, angle: 0.0, join_type: JoinType::Trunk }];

		// Generate split join points on the top disc using noise
		let split_noise = Perlin::new(self.config.seed + self.split_seed_offset);
		for i in 0..self.num_splits {
			// All splits are slightly below the top (0.95 instead of 1.0)
			let unit_position = 0.95;

			// Use noise to determine angle around trunk, evenly distributed
			let base_angle = (i as f32 / self.num_splits as f32) * 2.0 * std::f32::consts::PI;
			let angle_noise = split_noise.get([i as f64 * 0.3, 0.0]) as f32;
			let angle = base_angle + angle_noise * 0.2; // Small noise variation

			points.push(JoinPoint { unit_position, angle, join_type: JoinType::Trunk });
		}

		points
	}
}

impl Sdf for TrunkSplitSegment {
	fn distance(&self, p: Vec3) -> f32 {
		// Start with base trunk segment
		let mut dist = self.base_segment.distance(p);

		// Generate split join point bumps and union them
		let split_noise = Perlin::new(self.config.seed + self.split_seed_offset);
		for i in 0..self.num_splits {
			// All splits are slightly below the top
			let unit_position = 0.95;

			// Use noise to determine angle around trunk, evenly distributed
			let base_angle = (i as f32 / self.num_splits as f32) * 2.0 * std::f32::consts::PI;
			let angle_noise = split_noise.get([i as f64 * 0.3, 0.0]) as f32;
			let angle = base_angle + angle_noise * 0.2;

			// Calculate radius at this position (near top)
			let normalized_y = unit_position;
			let radius_at_pos = self.config.base_radius * (1.0 - normalized_y)
				+ self.config.top_radius * normalized_y;
			let bump_radius = radius_at_pos * 0.9; // Slightly within the radius

			// Create join point bump
			let join_bump = JoinPointSdf::new(
				unit_position,
				angle,
				bump_radius,
				0.1, // Bump height
				self.config.seed + self.split_seed_offset + i as u32,
				self.config.noise_amplitude,
				self.config.noise_frequency,
			);

			// Union with base segment
			dist = dist.min(join_bump.distance(p));
		}

		dist
	}
}
