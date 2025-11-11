pub mod affine;
pub mod extrusion;

use bevy::prelude::*;

/// Simple easing utilities
#[inline(always)]
fn smoothstep(t: f32) -> f32 {
	// clamp + cubic
	let t = t.clamp(0.0, 1.0);
	t * t * (3.0 - 2.0 * t)
}

/// 2D region types with fast signed distance φ(x,z).
pub enum Region2D {
	/// Axis-aligned rectangle with optional corner rounding.
	Rect { center: Vec2, half_extents: Vec2, round: f32 },
	/// Circle
	Circle { center: Vec2, radius: f32 },
	/// Convex polygon: precomputed outward unit edge normals and offsets.
	/// Distance φ(p) = max_i (dot(n_i, p) + b_i).
	ConvexPoly { normals: Vec<Vec2>, offsets: Vec<f32> }, // see builder below
}

impl Region2D {
	/// Factory for a convex polygon from CCW vertices (fast & robust).
	pub fn convex_from_ccw_vertices(verts: &[Vec2]) -> Self {
		assert!(verts.len() >= 3);
		let mut normals = Vec::with_capacity(verts.len());
		let mut offsets = Vec::with_capacity(verts.len());
		for i in 0..verts.len() {
			let a = verts[i];
			let b = verts[(i + 1) % verts.len()];
			let e = b - a;
			// CCW polygon => outward normal is (e.y, -e.x) normalized
			let n = Vec2::new(e.y, -e.x).normalize();
			let b_i = -n.dot(a);
			normals.push(n);
			offsets.push(b_i);
		}
		Region2D::ConvexPoly { normals, offsets }
	}

	/// Signed distance φ(x,z) (negative inside).
	#[inline(always)]
	pub fn sdf(&self, p: Vec2) -> f32 {
		match self {
			Region2D::Rect { center, half_extents, round } => {
				// Rounded rectangle SDF (2D) — cheap and stable
				let q = (p - *center).abs() - *half_extents + Vec2::splat(*round);
				let outside = q.max(Vec2::ZERO).length() - *round;
				let inside = q.x.max(q.y).min(0.0);
				outside + inside
			}
			Region2D::Circle { center, radius } => (p - *center).length() - *radius,
			Region2D::ConvexPoly { normals, offsets } => {
				// φ(p) = max_i (dot(n_i, p) + b_i)
				let mut m = -f32::INFINITY;
				for (n, b) in normals.iter().zip(offsets.iter()) {
					m = m.max(n.dot(p) + b);
				}
				m
			}
		}
	}
}
