pub mod affine;

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

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

/// Optional noise configuration for perturbing region boundaries
pub struct RegionNoise {
	/// The Perlin noise generator
	pub noise: Perlin,
	/// Noise frequency (controls the scale of noise sampling)
	pub frequency: f32,
	/// Noise amplitude (controls how much the boundary can be perturbed)
	/// Positive values allow the boundary to be pushed both inward and outward
	pub amplitude: f32,
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
		self.sdf_with_noise(p, None)
	}

	/// Signed distance with optional noise perturbation
	#[inline(always)]
	pub fn sdf_with_noise(&self, p: Vec2, noise: Option<&RegionNoise>) -> f32 {
		let mut d = match self {
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
		};

		// Apply noise perturbation to make the boundary wavy
		// The noise value is in [-1, 1], scaled by amplitude to allow both inward and outward perturbation
		if let Some(noise_config) = noise {
			let nval = noise_config.noise.get([
				p.x as f64 * noise_config.frequency as f64,
				p.y as f64 * noise_config.frequency as f64,
			]) as f32;
			d += nval * noise_config.amplitude;
		}

		d
	}
}
