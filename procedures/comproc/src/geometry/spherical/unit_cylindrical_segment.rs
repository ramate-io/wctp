use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::{
	mesh::{IdentifiedMesh, MeshId},
	NormalizeChunk,
};
use sdf::Sdf;

/// Simple trunk segment: noisy cylinder with trunk join points on top and bottom
#[derive(Debug, Clone)]
pub struct UnitCylindricalSegment {
	base_radius: f32,
	top_radius: f32,
}

impl UnitCylindricalSegment {
	pub fn new() -> Self {
		Self { base_radius: 0.5, top_radius: 0.4 }
	}
}

/// We should get the MeshBuilder trait for free since this is an SDF.
impl Sdf for UnitCylindricalSegment {
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
		let radius = self.base_radius * (1.0 - normalized_y) + self.top_radius * normalized_y;

		// Distance from center in XZ plane
		let xz_dist = (p.x * p.x + p.z * p.z).sqrt();

		// Base cylinder distance
		let mut dist = xz_dist - radius;

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

impl NormalizeChunk for UnitCylindricalSegment {
	fn normalize_chunk(&self, cascade_chunk: &CascadeChunk) -> CascadeChunk {
		CascadeChunk::unit_center_chunk().with_res_2(cascade_chunk.res_2)
	}
}

impl IdentifiedMesh for UnitCylindricalSegment {
	fn id(&self) -> MeshId {
		let debug_string = format!("{:?}", self);
		MeshId::new(debug_string)
	}
}
