use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::{
	mesh::{IdentifiedMesh, MeshId},
	NormalizeChunk,
};
use sdf::Sdf;

/// Noisy sphere: a sphere with Perlin noise perturbation for organic surface variation
#[derive(Debug, Clone)]
pub struct UnitBall {
	radius: f32,
}

impl UnitBall {
	pub fn new(radius: f32) -> Self {
		Self { radius }
	}
}

impl Sdf for UnitBall {
	/// Distance function for a noisy sphere
	/// The sphere is centered at the origin with configurable radius
	/// Perlin noise is used to perturb the surface for organic variation
	fn distance(&self, p: Vec3) -> f32 {
		// Distance from center
		let dist_from_center = p.length();

		// Base sphere distance (negative inside, positive outside)
		let dist = dist_from_center - self.radius;

		dist
	}
}

impl NormalizeChunk for UnitBall {
	fn normalize_chunk(&self, cascade_chunk: &CascadeChunk) -> CascadeChunk {
		CascadeChunk::unit_3d_center_chunk().with_res_2(cascade_chunk.res_2)
	}
}

impl IdentifiedMesh for UnitBall {
	fn id(&self) -> MeshId {
		let debug_string = format!("{:?}", self);
		MeshId::new(debug_string)
	}
}
