use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use super::{join_point::{JoinPoint, JoinType}, SegmentConfig};

/// Radial branches segment: trunk segment with n branch join points placed noisily
pub struct RadialBranchesSegment {
	config: SegmentConfig,
	noise: Perlin,
	/// Number of branch join points
	pub num_branches: usize,
	/// Seed offset for branch placement noise
	branch_seed_offset: u32,
}

impl RadialBranchesSegment {
	pub fn new(config: SegmentConfig, num_branches: usize) -> Self {
		let noise = Perlin::new(config.seed);
		Self {
			config,
			noise,
			num_branches,
			branch_seed_offset: 1000, // Offset to use different noise for branch placement
		}
	}

	/// Get join points: trunk join at bottom, n branch joins, trunk join at top
	pub fn join_points(&self) -> Vec<JoinPoint> {
		let mut points = vec![JoinPoint {
			unit_position: 0.0,
			angle: 0.0,
			join_type: JoinType::Trunk,
		}];

		// Generate branch join points using noise
		let branch_noise = Perlin::new(self.config.seed + self.branch_seed_offset);
		for i in 0..self.num_branches {
			// Use noise to determine position along segment (avoiding extremes)
			let pos_noise = branch_noise.get([i as f64 * 0.5, 0.0]) as f32;
			let unit_position = 0.2 + (pos_noise + 1.0) * 0.5 * 0.6; // Between 0.2 and 0.8

			// Use noise to determine angle around trunk
			let angle_noise = branch_noise.get([i as f64 * 0.7, 1.0]) as f32;
			let angle = (angle_noise + 1.0) * std::f32::consts::PI; // 0 to 2π

			points.push(JoinPoint {
				unit_position,
				angle,
				join_type: JoinType::Branch,
			});
		}

		points.push(JoinPoint {
			unit_position: 1.0,
			angle: 0.0,
			join_type: JoinType::Trunk,
		});

		points
	}

	/// Sample the unit SDF (same as SimpleTrunkSegment for now)
	pub fn unit_distance(&self, p: Vec3) -> f32 {
		let y = p.y.clamp(0.0, 1.0);
		let normalized_y = y;

		let radius = self.config.base_radius * (1.0 - normalized_y)
			+ self.config.top_radius * normalized_y;

		let xz_dist = (p.x * p.x + p.z * p.z).sqrt();
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
