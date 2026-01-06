use crate::noise::config::NoiseConfig;
use bevy::prelude::*;
use noise::{NoiseFn, Seedable};
use sdf::Sdf;

#[derive(Debug, Clone)]
pub struct Noisy<T: Sdf, N: NoiseFn<f64, 3> + Seedable> {
	pub sdf: T,
	pub noise_config: NoiseConfig<3, N>,
}

impl<T: Sdf, N: NoiseFn<f64, 3> + Seedable> Noisy<T, N> {
	pub fn new(sdf: T, noise_config: NoiseConfig<3, N>) -> Self {
		Self { sdf, noise_config }
	}
}

impl<T: Sdf, N: NoiseFn<f64, 3> + Seedable + Send + Sync> Sdf for Noisy<T, N> {
	fn distance(&self, p: Vec3) -> f32 {
		self.sdf.distance(p) + self.noise_config.vec3_amp(p) as f32
	}
}
