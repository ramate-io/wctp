use crate::sdf::perlin_terrain::ElevationModulation;
use crate::sdf::region_modulation::Region2D;
use bevy::prelude::*;

/// A unified modulation: applies both scaling (`a`) and offset (`b`) inside a smooth region.
/// Outside region → a = 1, b = 0
/// Inside region → a = inner_scale, b = extrusion_height
pub struct RegionAffineModulation {
	pub region: Region2D,
	pub inner_scale: f32,
	pub inner_offset: f32,
	pub inner_radius: f32,
	pub outer_radius: f32,
}

impl RegionAffineModulation {
	pub fn new(
		region: Region2D,
		inner_scale: f32,
		inner_offset: f32,
		inner_radius: f32,
		outer_radius: f32,
	) -> Self {
		Self {
			region,
			inner_scale,
			inner_offset,
			inner_radius,
			outer_radius: outer_radius.max(inner_radius + 0.001),
		}
	}

	#[inline(always)]
	fn smoothstep(t: f32) -> f32 {
		let t = t.clamp(0.0, 1.0);
		t * t * (3.0 - 2.0 * t)
	}

	#[inline(always)]
	fn region_weight(&self, p: Vec2) -> f32 {
		let d = self.region.sdf(p);
		if d < -self.inner_radius {
			0.0
		} else if d > self.outer_radius {
			1.0
		} else {
			let t = (d + self.inner_radius) / (self.inner_radius + self.outer_radius);
			Self::smoothstep(t)
		}
	}
}

impl ElevationModulation for RegionAffineModulation {
	fn modify_elevation(&self, elevation: f32, x: f32, z: f32) -> f32 {
		let p = Vec2::new(x, z);
		let w = self.region_weight(p);

		// Smooth blend between inside and outside values
		let a = self.inner_scale + (1.0 - self.inner_scale) * w;
		let b = self.inner_offset * (1.0 - w);

		a * elevation + b
	}
}
