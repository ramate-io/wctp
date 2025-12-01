use crate::Sdf;
use bevy::prelude::*;

pub struct TetrahedronSdf {
	pub vertices: [Vec3; 4],
}

impl Sdf for TetrahedronSdf {
	fn distance(&self, p: Vec3) -> f32 {
		// Signed distances to the 4 faces
		let mut max_dist = -f32::INFINITY;
		let v = &self.vertices;

		for (i0, i1, i2) in [(0, 1, 2), (0, 1, 3), (0, 2, 3), (1, 2, 3)] {
			let n = (v[i1] - v[i0]).cross(v[i2] - v[i0]).normalize();
			let d = (p - v[i0]).dot(n);
			max_dist = max_dist.max(d);
		}

		max_dist
	}
}
