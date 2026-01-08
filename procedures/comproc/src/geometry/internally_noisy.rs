use crate::noise::config::{InternalNoise, NoiseConfig};
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use noise::{NoiseFn, Seedable};
use render_item::{
	mesh::{IdentifiedMesh, MeshBuilder, MeshId},
	NormalizeChunk,
};

#[derive(Debug, Clone)]
pub struct InternallyNoisy<
	T: MeshBuilder + InternalNoise<D, N>,
	const D: usize,
	N: NoiseFn<f64, D> + Seedable + Clone,
> {
	mesh_builder: T,
	noise_config: NoiseConfig<D, N>,
}

impl<
		T: MeshBuilder + InternalNoise<D, N>,
		const D: usize,
		N: NoiseFn<f64, D> + Seedable + Clone,
	> InternallyNoisy<T, D, N>
{
	pub fn new(mut mesh_builder: T, noise_config: NoiseConfig<D, N>) -> Self {
		mesh_builder.set_internal_noise(noise_config.clone());
		Self { mesh_builder, noise_config }
	}
}

impl<
		T: MeshBuilder + InternalNoise<D, N>,
		const D: usize,
		N: NoiseFn<f64, D> + Seedable + Clone,
	> NormalizeChunk for InternallyNoisy<T, D, N>
{
	fn normalize_chunk(&self, cascade_chunk: &CascadeChunk) -> CascadeChunk {
		self.mesh_builder
			.normalize_chunk(cascade_chunk)
			.with_mu(self.noise_config.amplitude + 0.001)
	}
}

impl<
		T: MeshBuilder + InternalNoise<D, N>,
		const D: usize,
		N: NoiseFn<f64, D> + Seedable + Clone,
	> MeshBuilder for InternallyNoisy<T, D, N>
{
	fn build_mesh_impl(&self, cascade_chunk: &CascadeChunk) -> Option<Mesh> {
		self.mesh_builder.build_mesh_impl(cascade_chunk)
	}
}

// TODO: for some reason just requiring [IdentifiedMesh] isn't enough.
impl<
		T: MeshBuilder + IdentifiedMesh + InternalNoise<D, N>,
		const D: usize,
		N: NoiseFn<f64, D> + Seedable + Clone,
	> IdentifiedMesh for InternallyNoisy<T, D, N>
{
	fn id(&self) -> MeshId {
		self.mesh_builder.id()
	}
}
