use crate::sdf::region::RegionNoise;

use super::affine::RegionAffineModulation;
use super::Region2D;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

/// The idea here is to take a starting affine modulation region and permute out from it.
pub struct BranchingPlan {
	regions: Vec<RegionAffineModulation>,
	noise: Perlin,
	depth: usize,
}

impl BranchingPlan {
	pub fn new(base_region: RegionAffineModulation, noise: Perlin, depth: usize) -> Self {
		Self { regions: vec![base_region], noise, depth }
	}

	pub fn add_region(&mut self, region: RegionAffineModulation) {
		self.regions.push(region);
	}

	pub fn generate_regions(&self) -> Vec<RegionAffineModulation> {
		let mut total_regions = Vec::new();
		let mut last_regions = self.regions.clone();

		let fallback_noise =
			RegionNoise { noise: self.noise.clone(), amplitude: 1.0, frequency: 0.2 };

		for _ in 0..self.depth {
			let new_regions: Vec<RegionAffineModulation> = last_regions
				.iter()
				.map(|region| {
					let noise = region.noise.clone();
					let noise = noise.unwrap_or(fallback_noise.clone());
					region.branch_region(&noise)
				})
				.collect();
			total_regions.extend(new_regions.clone());
			last_regions = new_regions;
		}
		total_regions
	}
}
