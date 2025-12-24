use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::collections::{HashMap, HashSet, VecDeque};
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
	max_radius: f32,
	depth: usize,
	splitting_coefficient: f32,
	min_segment_length: f32,
	max_segment_length: f32,
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
			max_radius: 0.0,
			depth: 0,
			splitting_coefficient: 0.0,
			min_segment_length: 0.0,
			max_segment_length: 0.0,
		}
	}

	pub fn node_children_from(&self, position: Vec3) -> usize {
		// sample to get 0-1 value
		let sample =
			self.noise.get([position.x as f64, position.y as f64, position.z as f64]) as f32;

		// Map [-1,1] → [0,1]
		let sample = (sample * 0.5 + 0.5).clamp(0.0, 1.0);

		// floor sample/splitting_coefficient to get number of children
		let children = (sample / self.splitting_coefficient).floor() as usize;
		children
	}

	/// Gerenates a ray within the tolerance of the angle_tolerance
	pub fn unrestricted_ray_from(
		&self,
		position: Vec3,
		parent_ray: Vec3,
		child_index: usize,
	) -> Vec3 {
		let parent_dir = parent_ray.normalize();

		// Sample noise for deterministic angular variation
		let noise = self.noise.get([
			position.x as f64,
			position.y as f64,
			position.z as f64,
			child_index as f64,
		]) as f32;

		// Map noise [0,1] → [0, angle_tolerance]
		let max_angle = self.angle_tolerance.to_radians();
		let theta = noise.clamp(0.0, 1.0) * max_angle;

		// Random azimuth around the parent ray
		let phi = noise;

		// Build an orthonormal basis around parent_dir
		let up = if parent_dir.abs().y < 0.99 { Vec3::Y } else { Vec3::X };

		let tangent = parent_dir.cross(up).normalize();
		let bitangent = parent_dir.cross(tangent);

		// Direction inside the cone
		let direction = parent_dir * theta.cos()
			+ tangent * theta.sin() * phi.cos()
			+ bitangent * theta.sin() * phi.sin();

		direction.normalize()
	}

	/// Generates a ray within the angle and length tolerance
	pub fn ray_from(&self, position: Vec3, parent_ray: Vec3, child_index: usize) -> Vec3 {
		let direction = self.unrestricted_ray_from(position, parent_ray, child_index);

		// Independent noise for length
		let n_length = self.noise.get([
			position.x as f64,
			position.y as f64,
			position.z as f64,
			child_index as f64,
		]) as f32;

		// Map [-1,1] → [0,1]
		let n_length = (n_length * 0.5 + 0.5).clamp(0.0, 1.0);

		let length = self.min_segment_length
			+ n_length * (self.max_segment_length - self.min_segment_length);

		direction * length
	}

	pub fn radius_from(&self, position: Vec3, child_index: usize) -> f32 {
		let sample = self.noise.get([
			position.x as f64,
			position.y as f64,
			position.z as f64,
			child_index as f64,
		]) as f32;

		// Map [-1,1] → [0,1]
		let sample = (sample * 0.5 + 0.5).clamp(0.0, 1.0);

		let radius = self.min_radius + sample * (self.max_radius - self.min_radius);
		radius
	}

	pub fn build(&self) -> Branch {
		let mut branch = Branch::new();

		let initial_node = BranchNode::new(self.anchor, self.initial_radius);

		let mut queue = VecDeque::new();
		queue.push_back((initial_node.clone(), self.initial_ray.clone()));

		for _ in 0..self.depth {
			while let Some((node, ray)) = queue.pop_front() {
				let children = self.node_children_from(node.position);
				for i in 0..children {
					// generate the child attributes
					let child_ray = self.ray_from(node.position, ray, i);
					let child_position = node.position + child_ray;
					let child_radius = self.radius_from(node.position, i);
					let child_node = BranchNode::new(child_position, child_radius);

					// add the child to the branch and queue it for processing
					branch.add_child(node.clone(), child_node.clone());
					queue.push_back((child_node.clone(), child_ray));
				}

				// if we still haven't added the node, add it
				branch.add_node(node);
			}
		}

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
pub struct BranchSegment<'a> {
	pub start: &'a BranchNode,
	pub end: &'a BranchNode,
}

impl<'a> BranchSegment<'a> {
	pub fn ray(&self) -> Vec3 {
		(self.end.position - self.start.position).normalize()
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
		self.nodes.entry(parent).or_insert(HashSet::new()).insert(child);
	}

	pub fn nodes(&self) -> impl Iterator<Item = &BranchNode> {
		self.nodes.keys().collect::<Vec<&BranchNode>>().into_iter()
	}

	pub fn segments(&self) -> impl Iterator<Item = BranchSegment> {
		self.nodes
			.iter()
			.map(|(node, children)| {
				children.iter().map(|child| BranchSegment { start: node, end: child })
			})
			.flatten()
			.collect::<Vec<BranchSegment>>()
			.into_iter()
	}
}
