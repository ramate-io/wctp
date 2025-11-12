use crate::sdf::perlin_terrain::ElevationModulation;
use crate::sdf::region_modulation::{smoothstep, Region2D};
use bevy::prelude::*;

/// A generic, *easy-to-control* 2.5D extrusion:
/// height(x,z) = profile( φ(x,z) ), where φ is region SDF.
/// - Plateau inside by `inner`
/// - Zero outside by `outer`
/// - Smooth transition between using `smoothstep`
/// Optional FBM noise for roughness.
pub struct RegionExtrusion {
	// region SDF in XZ
	pub region: Region2D,
	// plateau height (positive = bump, negative = depression)
	pub height: f32,
	// inner/outer band widths controlling the slope thickness
	pub inner: f32, // >= 0 : inside band width
	pub outer: f32, // >= 0 : outside band width
}

impl RegionExtrusion {
	pub fn new(region: Region2D, height: f32, inner: f32, outer: f32) -> Self {
		Self { region, height, inner: inner.max(0.0), outer: outer.max(0.0) }
	}

	#[inline(always)]
	fn base_profile(&self, phi: f32) -> f32 {
		// φ <= -inner  -> plateau H
		// φ >=  outer  -> 0
		// transition in between with smoothstep
		if phi <= -self.inner {
			self.height
		} else if phi >= self.outer {
			0.0
		} else {
			// map φ from [-inner, outer] to [0,1] where 0=plateau, 1=outside
			let t = (phi + self.inner) / (self.inner + self.outer + f32::EPSILON);
			let w = smoothstep(t); // or use other easings
			self.height * (1.0 - w)
		}
	}
}

impl ElevationModulation for RegionExtrusion {
	#[inline(always)]
	fn modify_elevation(&self, elevation: f32, x: f32, z: f32) -> f32 {
		let p2 = Vec2::new(x, z);
		let phi = self.region.sdf(p2); // signed distance in XZ
		elevation + self.base_profile(phi)
	}
}
