use bevy::prelude::*;
use render_item::RenderItem;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// A marker trait for render items that can be used as partitions in a complex.
pub trait Partion: RenderItem + Hash + Debug + Clone {}

#[derive(Debug, Clone)]
pub struct PartitionCoordinates {
	pub start: Vec3,
	pub end: Vec3,
}

#[derive(Debug, Clone)]
pub struct PartitionComplex<P: Partion> {
	pub partitions: HashMap<PartitionCoordinates, P>,
}

/// A marker trait for floors in a complex.
pub trait Floor: RenderItem + Hash + Clone {}

#[derive(Debug, Clone)]
pub struct FloorCoordinates {
	pub position: Vec3,
}

#[derive(Debug, Clone)]
pub struct FloorComplex<F: Floor> {
	pub floors: HashMap<FloorCoordinates, F>,
}

#[derive(Debug, Clone)]
pub struct Complex<P: Partion, F: Floor> {
	pub partitions: PartitionComplex<P>,
	pub floors: FloorComplex<F>,
	pub anchor: Vec3,
	pub step_size: Vec3,
	/// NOTE: this representation makes it easy to build subcomplexes.
	pub step_count: (usize, usize, usize),
}

#[derive(Debug, Clone)]
pub enum ComplexElement<P: Partion, F: Floor> {
	Partition(P),
	Floor(F),
}

#[derive(Debug, Clone)]
pub enum ComplexCoordinates {
	Partition(PartitionCoordinates),
	Floor(FloorCoordinates),
}

#[derive(Debug, Clone)]
pub struct ComplexMember<P: Partion, F: Floor> {
	pub coordinates: ComplexCoordinates,
	pub element: ComplexElement<P, F>,
}

impl<P: Partion, F: Floor> Complex<P, F> {
	pub fn coords_iter(&self) -> impl Iterator<Item = ComplexCoordinates> {
		let mut members = Vec::new();

		for y in 0..self.step_count.1 {
			for z in 0..self.step_count.2 {
				// first do all of the floors
				for x in 0..self.step_count.0 {
					members.push(ComplexCoordinates::Floor(FloorCoordinates {
						position: self.anchor
							+ Vec3::new(
								x as f32 * self.step_size.x,
								y as f32 * self.step_size.y,
								z as f32 * self.step_size.z,
							),
					}));
				}

				// then do all of the partitions
				for x in 0..self.step_count.0 {
					// left-right
					members.push(ComplexCoordinates::Partition(PartitionCoordinates {
						start: self.anchor
							+ Vec3::new(
								x as f32 * self.step_size.x,
								y as f32 * self.step_size.y,
								z as f32 * self.step_size.z,
							),
						end: self.anchor
							+ Vec3::new(
								(x + 1) as f32 * self.step_size.x,
								(y) as f32 * self.step_size.y,
								(z) as f32 * self.step_size.z,
							),
					}));

					// up-down
					members.push(ComplexCoordinates::Partition(PartitionCoordinates {
						start: self.anchor
							+ Vec3::new(
								x as f32 * self.step_size.x,
								y as f32 * self.step_size.y,
								z as f32 * self.step_size.z,
							),
						end: self.anchor
							+ Vec3::new(
								(x) as f32 * self.step_size.x,
								(y) as f32 * self.step_size.y,
								(z + 1) as f32 * self.step_size.z,
							),
					}));
				}
			}
		}

		members.into_iter()
	}
}
