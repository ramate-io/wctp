use crate::{
	complex::{Complex, ComplexCoordinates, ComplexMember, Filler},
	meshes::walls::wall::Wall,
};
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone)]
pub struct NoiseConfig {
	scale: f32,
	noise: Perlin,
}

impl Default for NoiseConfig {
	fn default() -> Self {
		Self { scale: 0.1, noise: Perlin::new(42) }
	}
}

impl NoiseConfig {
	pub fn get(&self, position: Vec3) -> f32 {
		self.noise.get([
			position.x as f64 * self.scale as f64,
			position.y as f64 * self.scale as f64,
			position.z as f64 * self.scale as f64,
		]) as f32
	}

	pub fn get_on_unit_interval(&self, position: Vec3) -> f32 {
		let noise = self.get(position);
		noise * 0.5 + 0.5
	}
}

#[derive(Debug, Clone)]
pub struct ScratchpadFiller<T: Material> {
	noise_config: NoiseConfig,
	material: MeshMaterial3d<T>,
	floor_threshold: f32,
	partition_threshold: f32,
}

impl<T: Material> ScratchpadFiller<T> {
	pub fn new(material: MeshMaterial3d<T>) -> Self {
		Self {
			noise_config: NoiseConfig::default(),
			material,
			floor_threshold: 0.6,
			partition_threshold: 0.2,
		}
	}

	pub fn should_fill_floor(&self, position: Vec3) -> bool {
		self.noise_config.get_on_unit_interval(position) < self.floor_threshold
	}

	pub fn should_fill_partition(&self, position: Vec3) -> bool {
		self.noise_config.get_on_unit_interval(position) < self.partition_threshold
	}
}

impl<T: Material> Filler<Wall<T>, Wall<T>> for ScratchpadFiller<T> {
	fn fill(
		&mut self,
		complex: &mut Complex<Wall<T>, Wall<T>>,
		coordinates: ComplexCoordinates,
	) -> Option<ComplexMember<Wall<T>, Wall<T>>> {
		match coordinates {
			ComplexCoordinates::Floor(floor_coordinates) => {
				if self.should_fill_floor(floor_coordinates.position) {
					Some(ComplexMember::Floor(floor_coordinates, Wall::new(self.material.clone())))
				} else {
					None
				}
			}
			ComplexCoordinates::Partition(partition_coordinates) => {
				if self.should_fill_partition(partition_coordinates.start)
					&& !complex.partition_to_floors_below(&partition_coordinates).is_empty()
				{
					Some(ComplexMember::Partition(
						partition_coordinates,
						Wall::new(self.material.clone()),
					))
				} else {
					None
				}
			}
		}
	}
}
