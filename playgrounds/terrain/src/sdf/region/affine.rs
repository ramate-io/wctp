use crate::sdf::perlin_terrain::ElevationModulation;
use crate::sdf::region::{Region2D, RegionNoise};
use bevy::prelude::*;
use noise::NoiseFn;

/// A unified modulation: applies both scaling (`a`) and offset (`b`) inside a smooth region.
/// Outside region → a = 1, b = 0
/// Inside region → a = inner_scale, b = extrusion_height
#[derive(Debug, Clone)]
pub struct RegionAffineModulation {
	pub region: Region2D,
	pub inner_scale: f32,
	pub inner_offset: f32,
	pub inner_radius: f32,
	pub outer_radius: f32,
	/// Optional noise for perturbing the region boundary
	pub noise: Option<RegionNoise>,
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
			noise: None,
		}
	}

	/// Add noise perturbation to the region boundary
	pub fn with_noise(mut self, noise: RegionNoise) -> Self {
		self.noise = Some(noise);
		self
	}

	#[inline(always)]
	fn smoothstep(t: f32) -> f32 {
		let t = t.clamp(0.0, 1.0);
		t * t * (3.0 - 2.0 * t)
	}

	#[inline(always)]
	fn region_weight(&self, p: Vec2) -> f32 {
		let d = self.region.sdf_with_noise(p, self.noise.as_ref());
		if d < -self.inner_radius {
			0.0
		} else if d > self.outer_radius {
			1.0
		} else {
			let t = (d + self.inner_radius) / (self.inner_radius + self.outer_radius);
			Self::smoothstep(t)
		}
	}

	pub fn branch_region(&self, noise: &RegionNoise) -> Self {
		let new_region = self.region.branch_region(noise);

		// use noise to get the new inner scale
		let new_inner_scale = noise.noise.get([
			new_region.anchor_point_with_noise(noise).x as f64 * noise.frequency as f64,
			new_region.anchor_point_with_noise(noise).y as f64 * noise.frequency as f64,
		]) as f32;

		let new_inner_offset = noise.noise.get([
			new_region.anchor_point_with_noise(noise).x as f64 * noise.frequency as f64,
			new_region.anchor_point_with_noise(noise).y as f64 * noise.frequency as f64,
		]) as f32;

		let new_inner_radius = noise.noise.get([
			new_region.anchor_point_with_noise(noise).x as f64 * noise.frequency as f64,
			new_region.anchor_point_with_noise(noise).y as f64 * noise.frequency as f64,
		]) as f32;

		let new_outer_radius = noise.noise.get([
			new_region.anchor_point_with_noise(noise).x as f64 * noise.frequency as f64,
			new_region.anchor_point_with_noise(noise).y as f64 * noise.frequency as f64,
		]) as f32;

		Self {
			region: new_region,
			inner_scale: new_inner_scale,
			inner_offset: new_inner_offset,
			inner_radius: new_inner_radius,
			outer_radius: new_outer_radius,
			noise: Some(noise.clone()),
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
