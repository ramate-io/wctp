use bevy::prelude::*;
use render_item::RenderItem;
use std::collections::HashMap;
use std::hash::Hash;

/// A marker trait for render items that can be used as partitions in a complex.
pub trait Partion: RenderItem + Hash + Clone {}

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
	pub step_count: usize,
}
