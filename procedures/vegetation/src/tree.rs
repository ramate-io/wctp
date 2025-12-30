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

#[derive(Component, Clone)]
pub struct TreeRenderItem {
	tree_cache: HandleMap<SimpleTrunkSegment>,
	leaf_cache: HandleMap<NoisyBall>,
}

impl TreeRenderItem {
	pub fn new() -> Self {
		Self { tree_cache: HandleMap::new(), leaf_cache: HandleMap::new() }
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
		transform.translation - transform.rotation * (pivot_offset * Vec3::new(0.01, 0.01, 0.01))
	}

	pub fn spawn_trunk<M: Material>(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		material: MeshMaterial3d<M>,
	) {
		// Build tree segment dispatch
		let tree_segment = SimpleTrunkSegment::new(SegmentConfig::default());
		let mesh_handle = MeshHandle::new(tree_segment).with_handle_cache(self.tree_cache.clone());

		let centroid_anchor = self.centroid_anchor(transform);

		commands.spawn((
			CascadeChunk::unit_center_chunk().with_res_2(3),
			MeshDispatch::new(mesh_handle.clone()),
			Transform::from_translation(centroid_anchor + Vec3::new(0.0, 0.0, 0.0))
				.with_scale(Vec3::new(0.01, 0.01, 0.01)),
			MeshMaterial3d(material.0.clone()),
		));

		commands.spawn((
			CascadeChunk::unit_chunk().with_res_2(3),
			MeshDispatch::new(mesh_handle.clone()),
			Transform::from_translation(centroid_anchor + Vec3::new(0.003, 0.005, 0.004))
				.with_scale(Vec3::new(0.005, 0.005, 0.005))
				.with_rotation(Quat::from_rotation_arc(
					Vec3::new(1.0, 1.0, 1.0).normalize(),
					Vec3::Y,
				)),
			MeshMaterial3d(material.0.clone()),
		));

		commands.spawn((
			cascade_chunk.clone(),
			MeshDispatch::new(mesh_handle.clone()),
			Transform::from_translation(centroid_anchor + Vec3::new(0.0005, 0.0, 0.0005))
				.with_scale(Vec3::new(0.009, 0.02, 0.009)),
			MeshMaterial3d(material.0.clone()),
		));
	}

	pub fn spawn_branch<M: Material>(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		material: MeshMaterial3d<M>,
		initial_ray: Vec3,
	) {
		let mut branch_builder = BranchBuilder::common_tree_builder();

		// bias the branch towards the top
		branch_builder.angle_tolerance = 2.0;
		branch_builder.splitting_coefficient = 0.6;
		branch_builder.depth = 4;

		// anchor is on the ring of the trunk
		branch_builder.anchor = transform.translation + Vec3::new(0.0, 0.015, 0.0);

		// initial ray is sticking out to the side
		branch_builder.initial_ray = initial_ray;
		branch_builder.bias_ray = initial_ray + Vec3::new(0.0, 0.01, 0.0);
		branch_builder.bias_amount = 0.5;

		// min segment length is 0.002
		branch_builder.min_segment_length = 0.002;

		// max segment length is 0.004
		branch_builder.max_segment_length = 0.01;

		// min radius is 0.002
		branch_builder.min_radius = 0.001;

		// max radius is 0.004
		branch_builder.max_radius = 0.002;

		branch_builder.initial_radius = branch_builder.max_radius;

		let branch = branch_builder.build();

		// for now use the trunk segment
		let tree_segment = SimpleTrunkSegment::new(SegmentConfig::default());
		let mesh_handle = MeshHandle::new(tree_segment).with_handle_cache(self.tree_cache.clone());

		for segment in branch.segments() {
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
				MeshMaterial3d(material.0.clone()),
			));
		}

		for node in branch.nodes() {
			self.spawn_leaf_ball(commands, cascade_chunk, material.clone(), node.position);
		}
	}

	pub fn spawn_radial_branches<M: Material>(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		material: MeshMaterial3d<M>,
		branch_count: usize,
	) {
		for i in 0..branch_count {
			let angle = i as f32 * 2.0 * std::f32::consts::PI / branch_count as f32;
			let initial_ray =
				Vec3::new(angle.cos(), angle.sin() + angle.cos(), angle.sin()).normalize();
			self.spawn_branch(commands, cascade_chunk, transform, material.clone(), initial_ray);
		}
	}

	pub fn spawn_leaf_ball<M: Material>(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		material: MeshMaterial3d<M>,
		position: Vec3,
	) {
		// Build noisy ball mesh dispatch
		let noisy_ball = NoisyBall::new(NoisyBallConfig::default());
		let mesh_handle = MeshHandle::new(noisy_ball).with_handle_cache(self.leaf_cache.clone());

		// Spawn at the node position with appropriate scale
		let pivot_offset = Vec3::new(0.5, 0.5, 0.5);
		let scale = Vec3::splat(0.005);
		let translation = position - pivot_offset * scale;

		let ball_transform = Transform::from_translation(translation).with_scale(scale); // Scale for leaf ball size

		commands.spawn((
			cascade_chunk.clone(),
			MeshDispatch::new(mesh_handle),
			ball_transform,
			MeshMaterial3d(material.0.clone()),
		));
	}
}

impl RenderItem for TreeRenderItem {
	fn spawn_render_items<M: Material>(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		material: MeshMaterial3d<M>,
	) -> Vec<Entity> {
		self.spawn_trunk(commands, cascade_chunk, transform, material.clone());

		self.spawn_radial_branches(commands, cascade_chunk, transform, material, 10);

		vec![]
	}
}
