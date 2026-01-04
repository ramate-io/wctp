pub mod meshes;

use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use meshes::{
	canopy::{
		ball::{NoisyBall, NoisyBallConfig},
		branch::BranchBuilder,
	},
	trunk::segment::{SegmentConfig, SimpleTrunkSegment},
};
use render_item::{
	mesh::{cache::handle::map::HandleMap, handle::MeshHandle, MeshDispatch},
	RenderItem,
};

use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone)]
pub struct NoiseConfig {
	scale: f32,
	noise: Perlin,
}

impl Default for NoiseConfig {
	fn default() -> Self {
		Self { scale: 1000.0, noise: Perlin::new(0) }
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
		self.get(position) * 0.5 + 0.5
	}
}

#[derive(Component, Clone)]
pub struct TreeRenderItem<T: Material, L: Material> {
	tree_cache: HandleMap<SimpleTrunkSegment>,
	trunk_material: MeshMaterial3d<T>,
	leaf_cache: HandleMap<NoisyBall>,
	leaf_material: MeshMaterial3d<L>,

	height_scale: f32,

	// Segment assembly
	segement_configs: Vec<SegmentConfig>,

	// Foliage assembly
	foliage_configs: Vec<NoisyBallConfig>,

	// Branch assembly
	branch_min_segment_length: f32,
	branch_max_segment_length: f32,
	branch_min_radius: f32,
	branch_max_radius: f32,
	branch_count: usize,

	// Noise
	noise_config: NoiseConfig,
}

impl<T: Material, L: Material> TreeRenderItem<T, L> {
	pub fn new(trunk_material: MeshMaterial3d<T>, leaf_material: MeshMaterial3d<L>) -> Self {
		Self {
			tree_cache: HandleMap::new(),
			trunk_material,
			leaf_cache: HandleMap::new(),
			leaf_material,
			height_scale: 2.0,
			segement_configs: vec![SegmentConfig::default()],
			foliage_configs: vec![NoisyBallConfig::default()],
			branch_min_segment_length: 0.2,
			branch_max_segment_length: 1.0,
			branch_min_radius: 0.1,
			branch_max_radius: 0.2,
			noise_config: NoiseConfig::default(),
			branch_count: 10,
		}
	}

	pub fn with_tree_cache(mut self, tree_cache: HandleMap<SimpleTrunkSegment>) -> Self {
		self.tree_cache = tree_cache;
		self
	}

	pub fn with_leaf_cache(mut self, leaf_cache: HandleMap<NoisyBall>) -> Self {
		self.leaf_cache = leaf_cache;
		self
	}

	pub fn centroid_anchor(&self, transform: Transform) -> Vec3 {
		let pivot_offset = Vec3::new(0.5, 0.0, 0.5);
		transform.translation - transform.rotation * (pivot_offset * Vec3::new(1.0, 1.0, 1.0))
	}

	pub fn branch_segment_config(&self, index: usize) -> SegmentConfig {
		self.segement_configs[index % self.segement_configs.len()].clone()
	}

	pub fn branch_foliage_config(&self, index: usize) -> NoisyBallConfig {
		self.foliage_configs[index % self.foliage_configs.len()].clone()
	}

	pub fn spawn_trunk(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		material: MeshMaterial3d<T>,
	) {
		// Build tree segment dispatch
		let tree_segment = SimpleTrunkSegment::new(self.segement_configs[0].clone());
		let mesh_handle = MeshHandle::new(tree_segment).with_handle_cache(self.tree_cache.clone());

		let centroid_anchor = self.centroid_anchor(transform);

		commands.spawn((
			CascadeChunk::unit_center_chunk().with_res_2(3),
			MeshDispatch::new(mesh_handle.clone()),
			Transform::from_translation(centroid_anchor + Vec3::new(0.0, 0.0, 0.0))
				.with_scale(Vec3::new(1.0, self.height_scale / 2.0, 1.0)),
			MeshMaterial3d(material.0.clone()),
		));

		commands.spawn((
			CascadeChunk::unit_chunk().with_res_2(3),
			MeshDispatch::new(mesh_handle.clone()),
			Transform::from_translation(centroid_anchor + Vec3::new(0.0003, 0.0005, 0.0004))
				.with_scale(Vec3::new(0.5, self.height_scale / 4.0, 0.5))
				.with_rotation(Quat::from_rotation_arc(
					Vec3::new(1.0, 1.0, 1.0).normalize(),
					Vec3::Y,
				)),
			MeshMaterial3d(material.0.clone()),
		));

		commands.spawn((
			cascade_chunk.clone(),
			MeshDispatch::new(mesh_handle.clone()),
			Transform::from_translation(centroid_anchor).with_scale(Vec3::new(
				0.9,
				self.height_scale,
				0.9,
			)),
			MeshMaterial3d(material.0.clone()),
		));
	}

	pub fn branch_builder(&self, anchor: Vec3, initial_ray: Vec3) -> BranchBuilder {
		let mut branch_builder = BranchBuilder::common_tree_builder();
		branch_builder.anchor = anchor;
		branch_builder.initial_ray = initial_ray;
		branch_builder.bias_ray = initial_ray + Vec3::new(0.0, 0.01, 0.0);
		branch_builder.min_segment_length = self.branch_min_segment_length;
		branch_builder.max_segment_length = self.branch_max_segment_length;
		branch_builder.min_radius = self.branch_min_radius;
		branch_builder.max_radius = self.branch_max_radius;
		branch_builder
	}

	pub fn spawn_branch(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		height: f32,
		initial_ray: Vec3,
	) {
		let branch_builder =
			self.branch_builder(transform.translation + Vec3::new(0.0, height, 0.0), initial_ray);
		let branch = branch_builder.build();

		for (index, segment) in branch.segments().enumerate() {
			let segment_config = self.branch_segment_config(index);
			let tree_segment = SimpleTrunkSegment::new(segment_config);
			let mesh_handle =
				MeshHandle::new(tree_segment).with_handle_cache(self.tree_cache.clone());

			log::info!("Segment: {:?}", segment);
			let ray = segment.ray();
			let direction = ray.clone().normalize();
			let length = ray.length();

			let up = direction;

			// Pick a reference axis that is NOT parallel
			let reference = if up.abs_diff_eq(Vec3::Y, 1e-4) { Vec3::X } else { Vec3::Y };

			let right = up.cross(reference).normalize();
			let forward = right.cross(up);

			let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward));

			let pivot_offset = Vec3::new(0.5, 0.0, 0.5);
			let scale = Vec3::new(segment.start.radius, length, segment.start.radius);

			let transform = Transform {
				translation: segment.start.position - rotation * (pivot_offset * scale),
				rotation,
				scale,
			};

			log::info!("Transform: {:?}", transform);

			commands.spawn((
				cascade_chunk.clone(),
				MeshDispatch::new(mesh_handle.clone()),
				transform,
				MeshMaterial3d(self.trunk_material.0.clone()),
			));
		}

		for (index, node) in branch.nodes().enumerate() {
			self.spawn_leaf_ball(commands, cascade_chunk, node.position, index);
		}
	}

	pub fn get_branch_height(&self, last_position: Vec3) -> f32 {
		self.noise_config.get_on_unit_interval(last_position) * self.height_scale
	}

	pub fn spawn_radial_branches(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
	) {
		let pre_height = self.get_branch_height(transform.translation);
		let mut last_position = transform.translation + Vec3::new(0.0, pre_height, 0.0);

		for i in 0..self.branch_count {
			let height = self.get_branch_height(last_position);
			let angle = i as f32 * 2.0 * std::f32::consts::PI / self.branch_count as f32;
			let initial_ray =
				Vec3::new(angle.cos(), angle.sin() + angle.cos(), angle.sin()).normalize();
			self.spawn_branch(commands, cascade_chunk, transform, height, initial_ray);
			last_position = transform.translation + Vec3::new(0.0, height, 0.0);
		}
	}

	pub fn spawn_leaf_ball(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		position: Vec3,
		index: usize,
	) {
		// Build noisy ball mesh dispatch
		let noisy_ball = NoisyBall::new(self.branch_foliage_config(index));
		let mesh_handle = MeshHandle::new(noisy_ball).with_handle_cache(self.leaf_cache.clone());

		// Spawn at the node position with appropriate scale
		let pivot_offset = Vec3::new(0.5, 0.5, 0.5);
		let scale = Vec3::splat(0.5);
		let _translation = position - pivot_offset * scale;

		// spawn one on the point
		let ball_transform = Transform::from_translation(position).with_scale(scale); // Scale for leaf ball size
		commands.spawn((
			cascade_chunk.clone(),
			MeshDispatch::new(mesh_handle.clone()),
			ball_transform,
			MeshMaterial3d(self.leaf_material.0.clone()),
		));
	}
}

impl<T: Material, L: Material> RenderItem for TreeRenderItem<T, L> {
	fn spawn_render_items(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
	) -> Vec<Entity> {
		self.spawn_trunk(commands, cascade_chunk, transform, self.trunk_material.clone());

		self.spawn_radial_branches(commands, cascade_chunk, transform);

		vec![]
	}
}
