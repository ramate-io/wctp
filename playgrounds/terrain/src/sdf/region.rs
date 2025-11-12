pub mod affine;
pub mod branching;

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone)]
pub struct RectRegion {
	pub center: Vec2,
	pub half_extents: Vec2,
	pub round: f32,
}

#[derive(Debug, Clone)]
pub struct CircleRegion {
	pub center: Vec2,
	pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct ConvexPolyRegion {
	pub normals: Vec<Vec2>,
	pub offsets: Vec<f32>,
}

/// 2D region types with fast signed distance φ(x,z).
#[derive(Debug, Clone)]
pub enum Region2D {
	/// Axis-aligned rectangle with optional corner rounding.
	Rect(RectRegion),
	/// Circle
	Circle(CircleRegion),
	/// Convex polygon: precomputed outward unit edge normals and offsets.
	/// Distance φ(p) = max_i (dot(n_i, p) + b_i).
	ConvexPoly(ConvexPolyRegion), // see builder below
}

/// Optional noise configuration for perturbing region boundaries
#[derive(Debug, Clone)]
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
		Region2D::ConvexPoly(ConvexPolyRegion { normals, offsets })
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
			Region2D::Rect(RectRegion { center, half_extents, round }) => {
				// Rounded rectangle SDF (2D) — cheap and stable
				let q = (p - *center).abs() - *half_extents + Vec2::splat(*round);
				let outside = q.max(Vec2::ZERO).length() - *round;
				let inside = q.x.max(q.y).min(0.0);
				outside + inside
			}
			Region2D::Circle(CircleRegion { center, radius }) => (p - *center).length() - *radius,
			Region2D::ConvexPoly(ConvexPolyRegion { normals, offsets }) => {
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

	/// Gets the relative size of the region.
	pub fn relative_size(&self) -> f32 {
		match self {
			Region2D::Rect(RectRegion { half_extents, .. }) => half_extents.x,
			Region2D::Circle(CircleRegion { radius, .. }) => *radius,
			Region2D::ConvexPoly(ConvexPolyRegion { normals, .. }) => {
				let mut max_length = 0.0;
				for l in normals.iter().map(|n| n.length()) {
					if l > max_length {
						max_length = l;
					}
				}
				max_length
			}
		}
	}

	/// Gets the number of vertices for the convex poly.
	pub fn num_vertices(&self) -> usize {
		match self {
			Region2D::ConvexPoly(ConvexPolyRegion { normals, .. }) => normals.len(),
			_ => 0,
		}
	}

	/// Gets the anchor point for the given index.
	pub fn anchor_point(&self, index: usize) -> Vec2 {
		match self {
			// For rect it's always the center.
			Region2D::Rect(RectRegion { center, .. }) => *center,
			// For circle it's always the center.
			Region2D::Circle(CircleRegion { center, .. }) => *center,
			// For convex poly it's the vertex at the given index.
			Region2D::ConvexPoly(ConvexPolyRegion { normals, offsets }) => {
				normals[index] + offsets[index] * normals[index]
			}
		}
	}

	/// Gets the anchore point with noise for the given index.
	pub fn anchor_point_with_noise(&self, noise: &RegionNoise) -> Vec2 {
		// copmpute the index from noise value
		let index =
			(noise.noise.get([0.0, 0.0]) as f32 * self.num_vertices() as f32).floor() as usize;
		self.anchor_point(index)
	}

	/// Determines the size of the branching region.
	pub fn branching_size(&self, noise: &RegionNoise) -> f32 {
		let anchor = self.anchor_point_with_noise(noise);
		let size = noise.noise.get([
			anchor.x as f64 * noise.frequency as f64,
			anchor.y as f64 * noise.frequency as f64,
		]) as f32;
		size * self.relative_size()
	}

	/// Scales the region by the given factor.
	pub fn scale(&self, factor: f32) -> Self {
		match self {
			Region2D::Rect(rect_region) => Region2D::Rect(RectRegion {
				half_extents: rect_region.half_extents * factor,
				..rect_region.clone()
			}),
			Region2D::Circle(circle_region) => Region2D::Circle(CircleRegion {
				radius: circle_region.radius * factor,
				..circle_region.clone()
			}),
			Region2D::ConvexPoly(convex_poly_region) => Region2D::ConvexPoly(ConvexPolyRegion {
				normals: convex_poly_region.normals.iter().map(|n| n * factor).collect(),
				offsets: convex_poly_region.offsets.iter().map(|o| o * factor).collect(),
				..convex_poly_region.clone()
			}),
		}
	}

	/// Reacnhors the region to the given anchor point.
	pub fn reanchor(&self, anchor: Vec2) -> Self {
		match self {
			Region2D::Rect(rect_region) => {
				Region2D::Rect(RectRegion { center: anchor, ..rect_region.clone() })
			}
			Region2D::Circle(circle_region) => {
				Region2D::Circle(CircleRegion { center: anchor, ..circle_region.clone() })
			}
			Region2D::ConvexPoly(convex_poly_region) => Region2D::convex_from_ccw_vertices(
				&convex_poly_region.normals.iter().map(|n| n + anchor).collect::<Vec<Vec2>>(),
			),
		}
	}

	/// Takes some noise and creates a new affine region.
	pub fn branch_region(&self, noise: &RegionNoise) -> Self {
		// sample the noise to determine which shape to use
		self.scale(self.branching_size(noise))
			.reanchor(self.anchor_point_with_noise(noise))
	}
}
