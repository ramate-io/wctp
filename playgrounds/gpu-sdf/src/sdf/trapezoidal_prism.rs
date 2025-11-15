use crate::sdf::Sdf;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

/// Trapezoidal (frustum-shaped) prism SDF.
/// Top and bottom are rectangles that can differ in size,
/// forming slanted sides along Y.
pub struct TrapezoidalPrismSdf {
	pub center: Vec3,      // midpoint between top and bottom
	pub size_bottom: Vec2, // half-size at y = -height/2
	pub size_top: Vec2,    // half-size at y = +height/2
	pub height: f32,       // total height
	pub noise: Option<Perlin>,
	pub noise_factor: f32,
}

impl TrapezoidalPrismSdf {
	/// Create a new trapezoidal prism SDF
	pub fn new(center: Vec3, size_bottom: Vec2, size_top: Vec2, height: f32) -> Self {
		Self { center, size_bottom, size_top, height, noise: None, noise_factor: 0.0 }
	}

	/// Add noise perturbation to the surface
	pub fn with_noise(mut self, noise: Perlin) -> Self {
		self.noise = Some(noise);
		self
	}

	/// Set the noise factor (amplitude of perturbation)
	pub fn with_noise_factor(mut self, factor: f32) -> Self {
		self.noise_factor = factor;
		self
	}

	/// Compute noise perturbation for surface roughness
	fn compute_noise(&self, p: Vec3) -> f32 {
		if let Some(noise) = &self.noise {
			let nval = noise.get([
				p.x as f64 * self.noise_factor as f64,
				p.y as f64 * self.noise_factor as f64,
				p.z as f64 * self.noise_factor as f64,
			]) as f32;
			nval * self.noise_factor
		} else {
			0.0
		}
	}
}

impl Sdf for TrapezoidalPrismSdf {
	#[inline(always)]
	fn distance(&self, p: Vec3) -> f32 {
		// Local coordinates relative to center
		let q = p - self.center;
		let half_h = self.height * 0.5;

		// Determine Y position and which cross-section to use
		let (t, y_dist_to_cap) = if q.y < -half_h {
			// Below bottom - use bottom cross-section
			(0.0, -half_h - q.y)
		} else if q.y > half_h {
			// Above top - use top cross-section
			(1.0, q.y - half_h)
		} else {
			// Within Y bounds - interpolate
			((q.y + half_h) / self.height, 0.0)
		};

		// Interpolate half-extents in XZ plane at this Y level
		let hxz = self.size_bottom.lerp(self.size_top, t);

		// Compute distance to XZ cross-section
		let d_xz = Vec2::new(q.x.abs(), q.z.abs()) - hxz;

		// Check if inside XZ cross-section
		let inside_xz = d_xz.x < 0.0 && d_xz.y < 0.0;
		let inside_y = q.y >= -half_h && q.y <= half_h;

		// Compute base SDF distance
		if inside_xz && inside_y {
			// Inside the prism - return maximum negative distance
			let dist_xz = d_xz.x.max(d_xz.y);
			let dist_y = (-half_h - q.y).max(q.y - half_h);
			let base_dist = dist_xz.max(dist_y);

			base_dist * self.compute_noise(p)
		} else {
			// Outside - compute distance to nearest surface
			let outside_xz = if inside_xz { 0.0 } else { d_xz.max(Vec2::ZERO).length() };

			if inside_xz {
				// Inside XZ but outside Y - distance to cap
				y_dist_to_cap
			} else if inside_y {
				// Inside Y but outside XZ - distance to side
				outside_xz
			} else {
				// Outside in both - combine distances
				Vec2::new(outside_xz, y_dist_to_cap).length()
			}
		}
	}
}
