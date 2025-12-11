use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use super::{join_point::{JoinPoint, JoinType}, SegmentConfig};

/// Trunk split segment: trunk segment with n trunk join points on the top disc
/// The trunk draws into these join points (tapers toward them)
pub struct TrunkSplitSegment {
	config: SegmentConfig,
	noise: Perlin,
	/// Number of trunk split points
	pub num_splits: usize,
	/// Seed offset for split placement noise
	split_seed_offset: u32,
}

impl TrunkSplitSegment {
	pub fn new(config: SegmentConfig, num_splits: usize) -> Self {
		let noise = Perlin::new(config.seed);
		Self {
			config,
			noise,
			num_splits,
			split_seed_offset: 2000, // Offset for split placement noise
		}
	}

	/// Get join points: trunk join at bottom, n trunk joins at top
	pub fn join_points(&self) -> Vec<JoinPoint> {
		let mut points = vec![JoinPoint {
			unit_position: 0.0,
			angle: 0.0,
			join_type: JoinType::Trunk,
		}];

		// Generate split join points on the top disc using noise
		let split_noise = Perlin::new(self.config.seed + self.split_seed_offset);
		for i in 0..self.num_splits {
			// All splits are at the top (1.0)
			let unit_position = 1.0;

			// Use noise to determine angle around trunk, evenly distributed
			let base_angle = (i as f32 / self.num_splits as f32) * 2.0 * std::f32::consts::PI;
			let angle_noise = split_noise.get([i as f64 * 0.3, 0.0]) as f32;
			let angle = base_angle + angle_noise * 0.2; // Small noise variation

			points.push(JoinPoint {
				unit_position,
				angle,
				join_type: JoinType::Trunk,
			});
		}

		points
	}

	/// Sample the unit SDF with trunk drawing into split points
	pub fn unit_distance(&self, p: Vec3) -> f32 {
		let y = p.y.clamp(0.0, 1.0);
		let normalized_y = y;

		// Base radius interpolation
		let mut radius = self.config.base_radius * (1.0 - normalized_y)
			+ self.config.top_radius * normalized_y;

		// Calculate angle in XZ plane
		let xz_angle = p.z.atan2(p.x);
		let xz_dist = (p.x * p.x + p.z * p.z).sqrt();

		// Get split join points to draw toward
		let split_noise = Perlin::new(self.config.seed + self.split_seed_offset);
		let mut min_split_dist = f32::INFINITY;

		for i in 0..self.num_splits {
			let base_angle = (i as f32 / self.num_splits as f32) * 2.0 * std::f32::consts::PI;
			let angle_noise = split_noise.get([i as f64 * 0.3, 0.0]) as f32;
			let split_angle = base_angle + angle_noise * 0.2;

			// Distance from this split point's direction
			let angle_diff = ((xz_angle - split_angle + std::f32::consts::PI)
				% (2.0 * std::f32::consts::PI))
				- std::f32::consts::PI;
			let angle_dist = angle_diff.abs();

			// As we approach the top, draw toward split points
			let draw_factor = normalized_y.powi(2); // Stronger near top
			let split_radius = radius * (1.0 - draw_factor * 0.5 * (1.0 - angle_dist / std::f32::consts::PI));

			min_split_dist = min_split_dist.min(split_radius);
		}

		// Blend between base radius and split-drawn radius
		if min_split_dist < f32::INFINITY {
			let blend = normalized_y.powi(2);
			radius = radius * (1.0 - blend) + min_split_dist * blend;
		}

		let mut dist = xz_dist - radius;

		// Add noise perturbation
		let noise_value = self.noise.get([
			p.x as f64 * self.config.noise_frequency as f64,
			y as f64 * self.config.noise_frequency as f64,
			p.z as f64 * self.config.noise_frequency as f64,
		]) as f32;
		dist += noise_value * self.config.noise_amplitude;

		// Handle end caps
		if y < 0.0 {
			dist = dist.max(-y);
		} else if y > 1.0 {
			dist = dist.max(y - 1.0);
		}

		dist
	}
}
