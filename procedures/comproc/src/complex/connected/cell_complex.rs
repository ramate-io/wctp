use bevy::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub trait Vertex: Hash + Eq + Clone + Debug {
	fn point(&self) -> Vec3;
}
pub trait Edge<O, V: Vertex>: Hash + Eq + Clone + Debug {
	fn vertices(&self, on: &O) -> Vec<V>;
}
pub trait Face<O, V: Vertex, E: Edge<O, V>>: Hash + Eq + Clone + Debug {
	fn edges(&self, on: &O) -> Vec<E>;
}

pub struct CellComplex3d<V, E, F> {
	pub v_to_e: HashMap<V, E>,
	pub e_to_f: HashMap<E, F>,
	pub f_to_v: HashMap<F, V>,
}
