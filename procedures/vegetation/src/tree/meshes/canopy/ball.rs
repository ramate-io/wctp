use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use noise::{NoiseFn, Perlin};
use render_item::{
	mesh::{IdentifiedMesh, MeshId},
	NormalizeChunk,
};
use sdf::Sdf;

/// Configuration for a noisy sphere/ball
/// All balls work in unit space (0-1) and are transformed later
#[derive(Debug, Clone)]
pub struct NoisyBallConfig {
	/// Seed for noise generation
	pub seed: u32,
	/// Base radius of the sphere (in unit space, typically 0.5)
	pub radius: f32,
	/// Noise amplitude for surface variation
	/// Higher values create more pronounced surface bumps
	pub noise_amplitude: f32,
	/// Noise frequency for surface variation
	/// Higher values create finer, more detailed noise patterns
	pub noise_frequency: f32,
	/// Number of noise octaves for fractal detail
	/// More octaves = more detailed but potentially slower
	pub noise_octaves: u32,
}

impl Default for NoisyBallConfig {
	fn default() -> Self {
		Self { seed: 0, radius: 0.5, noise_amplitude: 0.1, noise_frequency: 3.0, noise_octaves: 3 }
	}
}

/// Noisy sphere: a sphere with Perlin noise perturbation for organic surface variation
#[derive(Debug, Clone)]
pub struct NoisyBall {
	config: NoisyBallConfig,
	noise: Perlin,
}

impl NoisyBall {
	pub fn new(config: NoisyBallConfig) -> Self {
		let noise = Perlin::new(config.seed);
		Self { config, noise }
	}
}

impl Sdf for NoisyBall {
	/// Distance function for a noisy sphere
	/// The sphere is centered at the origin with configurable radius
	/// Perlin noise is used to perturb the surface for organic variation
	fn distance(&self, p: Vec3) -> f32 {
		// Distance from center
		let dist_from_center = p.length();

		// Base sphere distance (negative inside, positive outside)
		let mut dist = dist_from_center - self.config.radius;

		// Add noise perturbation for surface variation
		// Sample noise at the point's position, scaled by frequency
		let noise_value = self.noise.get([
			p.x as f64 * self.config.noise_frequency as f64,
			p.y as f64 * self.config.noise_frequency as f64,
			p.z as f64 * self.config.noise_frequency as f64,
		]) as f32;

		// Apply noise amplitude to the distance
		// This creates bumps and indentations on the sphere surface
		dist += noise_value * self.config.noise_amplitude;

		dist
	}
}

impl NormalizeChunk for NoisyBall {
	fn normalize_chunk(&self, cascade_chunk: &CascadeChunk) -> CascadeChunk {
		CascadeChunk::unit_3d_center_chunk()
			.with_res_2(cascade_chunk.res_2)
			.with_mu(self.config.noise_amplitude + 0.001)
	}
}

impl IdentifiedMesh for NoisyBall {
	fn id(&self) -> MeshId {
		let debug_string = format!("{:?}", self);
		MeshId::new(debug_string)
	}
}
