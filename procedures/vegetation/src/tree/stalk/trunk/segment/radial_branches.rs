use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use sdf::Sdf;
use super::{
	join_point::{JoinPoint, JoinPointSdf, JoinType},
	simple::SimpleTrunkSegment,
	SegmentConfig,
};

/// Radial branches segment: trunk segment with n branch join points placed noisily
/// Union of simple trunk segment with join point bumps placed slightly within the radius
pub struct RadialBranchesSegment {
	base_segment: SimpleTrunkSegment,
	num_branches: usize,
	config: SegmentConfig,
	branch_seed_offset: u32,
}

impl RadialBranchesSegment {
	pub fn new(config: SegmentConfig, num_branches: usize) -> Self {
		Self {
			base_segment: SimpleTrunkSegment::new(config.clone()),
			num_branches,
			config,
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
}

impl Sdf for RadialBranchesSegment {
	fn distance(&self, p: Vec3) -> f32 {
		// Start with base trunk segment
		let mut dist = self.base_segment.distance(p);

		// Generate branch join point segments and union them
		let branch_noise = Perlin::new(self.config.seed + self.branch_seed_offset);
		for i in 0..self.num_branches {
			// Use noise to determine position along segment
			let pos_noise = branch_noise.get([i as f64 * 0.5, 0.0]) as f32;
			let unit_position = 0.2 + (pos_noise + 1.0) * 0.5 * 0.6;

			// Use noise to determine angle around trunk
			let angle_noise = branch_noise.get([i as f64 * 0.7, 1.0]) as f32;
			let angle = (angle_noise + 1.0) * std::f32::consts::PI;

			// Create join point segment (SimpleTrunkSegment angled in 3D)
			// Use upward_component of 0.0 for branches (horizontal)
			let join_segment = JoinPointSdf::new(
				unit_position,
				angle,
				self.config.clone(),
				self.branch_seed_offset + i as u32,
				0.0,
			);

			// Union with base segment
			dist = dist.min(join_segment.distance(p));
		}

		dist
	}
}
