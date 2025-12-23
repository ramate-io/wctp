use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug, Clone)]
pub struct BranchBuilder {
	noise: Perlin,
	anchor: Vec3,
	initial_ray: Vec3,
	angle_tolerance: f32,
	initial_radius: f32,
	min_radius: f32,
	depth: usize,
	splitting_coefficient: f32,
}

impl BranchBuilder {
	pub fn new() -> Self {
		Self {
			noise: Perlin::new(0),
			anchor: Vec3::ZERO,
			initial_ray: Vec3::ZERO,
			angle_tolerance: 0.0,
			initial_radius: 0.0,
			min_radius: 0.0,
			depth: 0,
			splitting_coefficient: 0.0,
		}
	}

	pub fn node_children_from(&self, position: Vec3) -> usize {
		// sample to get 0-1 value
		let sample =
			self.noise.get([position.x as f64, position.y as f64, position.z as f64]) as f32;

		// floor sample/splitting_coefficient to get number of children
		let children = (sample / self.splitting_coefficient).floor() as usize;
		children
	}

	pub fn build(&self) -> Branch {
		let mut branch = Branch::new();
		branch.add_node(BranchNode::new(self.anchor, self.initial_radius));
		branch
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct BranchNode {
	pub position: Vec3,
	pub radius: f32,
}

impl Eq for BranchNode {}

impl Hash for BranchNode {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.position.x.to_bits().hash(state);
		self.position.y.to_bits().hash(state);
		self.position.z.to_bits().hash(state);
		self.radius.to_bits().hash(state);
	}
}

impl BranchNode {
	pub fn new(position: Vec3, radius: f32) -> Self {
		Self { position, radius }
	}
}

#[derive(Debug, Clone)]
pub struct Branch {
	nodes: HashMap<BranchNode, HashSet<BranchNode>>,
}

impl Branch {
	fn new() -> Self {
		Self { nodes: HashMap::new() }
	}

	fn add_node(&mut self, node: BranchNode) {
		self.nodes.insert(node, HashSet::new());
	}

	fn add_child(&mut self, parent: BranchNode, child: BranchNode) {
		self.nodes.get_mut(&parent).unwrap().insert(child);
	}
}
