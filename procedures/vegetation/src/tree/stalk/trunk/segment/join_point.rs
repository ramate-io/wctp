use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use sdf::Sdf;

/// A join point where another segment can attach
#[derive(Debug, Clone)]
pub struct JoinPoint {
	/// Position in unit space (0-1 along the segment, 0 = bottom, 1 = top)
	pub unit_position: f32,
	/// Angle around the trunk axis in radians (0 = +X direction)
	pub angle: f32,
	/// Type of join point
	pub join_type: JoinType,
}

/// Type of join point
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinType {
	/// Join point for another trunk segment
	Trunk,
	/// Join point for a branch
	Branch,
}

/// SDF for a join point - a short noisy cylinder bump/apron
pub struct JoinPointSdf {
	/// Center position in unit space (y coordinate)
	unit_y: f32,
	/// Angle around trunk axis
	angle: f32,
	/// Radius of the join bump (slightly within the trunk radius)
	bump_radius: f32,
	/// Height of the bump
	bump_height: f32,
	/// Noise for surface variation
	noise: Perlin,
	/// Noise amplitude
	noise_amplitude: f32,
	/// Noise frequency
	noise_frequency: f32,
}

impl JoinPointSdf {
	pub fn new(
		unit_y: f32,
		angle: f32,
		bump_radius: f32,
		bump_height: f32,
		seed: u32,
		noise_amplitude: f32,
		noise_frequency: f32,
	) -> Self {
		Self {
			unit_y,
			angle,
			bump_radius,
			bump_height,
			noise: Perlin::new(seed),
			noise_amplitude,
			noise_frequency,
		}
	}
}

impl Sdf for JoinPointSdf {
	fn distance(&self, p: Vec3) -> f32 {
		// Transform point to join point's local space
		// Rotate around Y axis by the join angle
		let cos_a = self.angle.cos();
		let sin_a = self.angle.sin();
		let local_x = p.x * cos_a - p.z * sin_a;
		let local_z = p.x * sin_a + p.z * cos_a;

		// Translate y to be relative to join point center
		let local_y = p.y - self.unit_y;

		// Distance from center in XZ plane
		let xz_dist = (local_x * local_x + local_z * local_z).sqrt();

		// Base cylinder distance
		let mut dist = xz_dist - self.bump_radius;

		// Add noise perturbation
		let noise_value = self.noise.get([
			local_x as f64 * self.noise_frequency as f64,
			local_y as f64 * self.noise_frequency as f64,
			local_z as f64 * self.noise_frequency as f64,
		]) as f32;
		dist += noise_value * self.noise_amplitude;

		// Handle end caps (bump height)
		let half_height = self.bump_height / 2.0;
		let cap_dist = (local_y.abs() - half_height).max(0.0);

		// Combine distances (capsule logic)
		let outside_dist = (dist.max(0.0).powi(2) + cap_dist.powi(2)).sqrt();
		let inside_dist = dist.max(cap_dist);

		if inside_dist < 0.0 {
			inside_dist
		} else {
			outside_dist
		}
	}
}
