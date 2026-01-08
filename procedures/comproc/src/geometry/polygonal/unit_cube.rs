use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::{
	mesh::{IdentifiedMesh, MeshId},
	NormalizeChunk,
};
use sdf::Sdf;

/// Noisy sphere: a sphere with Perlin noise perturbation for organic surface variation
#[derive(Debug, Clone)]
pub struct UnitCube {
	radius: f32,
}

impl UnitCube {
	pub fn new(radius: f32) -> Self {
		Self { radius }
	}
}

impl Sdf for UnitCube {
	/// Distance function for a unit cube
	fn distance(&self, p: Vec3) -> f32 {
		// Distance from center
		p.x.abs().max(p.y.abs()).max(p.z.abs()) - self.radius
	}
}

impl NormalizeChunk for UnitCube {
	fn normalize_chunk(&self, cascade_chunk: &CascadeChunk) -> CascadeChunk {
		CascadeChunk::unit_3d_center_chunk().with_res_2(cascade_chunk.res_2)
	}
}

impl IdentifiedMesh for UnitCube {
	fn id(&self) -> MeshId {
		let debug_string = format!("{:?}", self);
		MeshId::new(debug_string)
	}
}
