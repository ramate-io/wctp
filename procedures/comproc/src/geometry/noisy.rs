use crate::noise::config::NoiseConfig;
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use noise::{NoiseFn, Seedable};
use render_item::{
	mesh::{IdentifiedMesh, MeshId},
	NormalizeChunk,
};
use sdf::Sdf;
use std::fmt::Debug;

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

impl<T: Sdf + IdentifiedMesh, N: NoiseFn<f64, 3> + Seedable + Send + Sync> IdentifiedMesh
	for Noisy<T, N>
{
	fn id(&self) -> MeshId {
		self.sdf.id().with_suffix(&format!("{:?}", self.noise_config))
	}
}

impl<T: Sdf + NormalizeChunk, N: NoiseFn<f64, 3> + Seedable + Send + Sync> NormalizeChunk
	for Noisy<T, N>
{
	fn normalize_chunk(&self, cascade_chunk: &CascadeChunk) -> CascadeChunk {
		self.sdf
			.normalize_chunk(cascade_chunk)
			.with_mu(self.noise_config.amplitude + 0.001)
	}
}
