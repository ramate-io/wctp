use crate::region::RegionNoise;

use super::affine::RegionAffineModulation;
use bevy::prelude::*;
use noise::{Perlin, Seedable};

/// The idea here is to take a starting affine modulation region and permute out from it.
pub struct BranchingPlan {
	regions: Vec<RegionAffineModulation>,
	noise: Perlin,
	depth: usize,
	breadth: usize,
}

impl BranchingPlan {
	pub fn new(
		base_region: RegionAffineModulation,
		noise: Perlin,
		depth: usize,
		breadth: usize,
	) -> Self {
		Self { regions: vec![base_region], noise, depth, breadth }
	}

	pub fn add_region(&mut self, region: RegionAffineModulation) {
		self.regions.push(region);
	}

	pub fn generate_regions(&self) -> Vec<RegionAffineModulation> {
		let mut total_regions = Vec::new();
		let mut last_regions = self.regions.clone();

		let fallback_noise =
			RegionNoise { noise: self.noise.clone(), amplitude: 1.0, frequency: 0.2 };

		for i in 0..self.depth {
			let new_regions: Vec<RegionAffineModulation> = last_regions
				.iter()
				.enumerate()
				.map(|(j, region)| {
					let mut new_regions = Vec::new();
					for k in 0..self.breadth {
						let noise = region.noise.clone();
						let mut noise = noise.unwrap_or(fallback_noise.clone());
						noise.noise = noise
							.noise
							.set_seed(noise.noise.seed() + (i * j * k + i + j + k) as u32);
						let new_region = region.branch_region(&noise);
						new_regions.push(new_region);
					}
					new_regions
				})
				.collect::<Vec<Vec<RegionAffineModulation>>>()
				.into_iter()
				.flatten()
				.collect();
			total_regions.extend(new_regions.clone());
			last_regions = new_regions;
		}
		total_regions
	}
}
