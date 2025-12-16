use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use render_item::{
	cache::handle_cache::HandleCache,
	mesh::{MeshBuilder, MeshCache, MeshFetcher, MeshHandleCache},
	NormalizeChunk,
};
use sdf::Sdf;

/// Base configuration for a trunk segment
/// All segments work in unit space (0-1) and are transformed later
#[derive(Clone)]
pub struct SegmentConfig {
	/// Seed for noise generation
	pub seed: u32,
	/// Base radius at bottom (in unit space, typically 0.5)
	pub base_radius: f32,
	/// Top radius at top (in unit space, typically 0.4)
	pub top_radius: f32,
	/// Noise amplitude for surface variation
	pub noise_amplitude: f32,
	/// Noise frequency for surface variation
	pub noise_frequency: f32,
}

impl Default for SegmentConfig {
	fn default() -> Self {
		Self {
			seed: 0,
			base_radius: 0.5,
			top_radius: 0.4,
			noise_amplitude: 0.05,
			noise_frequency: 5.0,
		}
	}
}

/// Simple trunk segment: noisy cylinder with trunk join points on top and bottom
#[derive(Clone)]
pub struct SimpleTrunkSegment {
	config: SegmentConfig,
	noise: Perlin,
}

impl SimpleTrunkSegment {
	pub fn new(config: SegmentConfig) -> Self {
		let noise = Perlin::new(config.seed);
		Self { config, noise }
	}
}

impl Sdf for SimpleTrunkSegment {
	/// NOTE: early on there appeared to be a  bug that gives this some slightly weird sharp facets.
	/// By playing with chunk settings, it was possible to make facets disappear,
	/// suggesting this was actually an LOD issue.
	///
	/// If such a bug reappears, we should investigate further.
	///
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
