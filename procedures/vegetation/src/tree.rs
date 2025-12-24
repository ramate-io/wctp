pub mod meshes;

use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use meshes::{
	canopy::branch::BranchBuilder,
	trunk::segment::{SegmentConfig, SimpleTrunkSegment},
};
use render_item::{
	mesh::{cache::handle::map::HandleMap, handle::MeshHandle, MeshDispatch},
	RenderItem,
};

#[derive(Component, Clone)]
pub struct TreeRenderItem {
	tree_cache: HandleMap<SimpleTrunkSegment>,
}

impl TreeRenderItem {
	pub fn new() -> Self {
		Self { tree_cache: HandleMap::new() }
	}

	pub fn with_tree_cache(mut self, tree_cache: HandleMap<SimpleTrunkSegment>) -> Self {
		self.tree_cache = tree_cache;
		self
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

		commands.spawn((
			CascadeChunk::unit_center_chunk().with_res_2(3),
			MeshDispatch::new(mesh_handle.clone()),
			Transform::from_translation(transform.translation + Vec3::new(0.0, 0.0, 0.0))
				.with_scale(Vec3::new(0.01, 0.01, 0.01)),
			MeshMaterial3d(material.0.clone()),
		));

		commands.spawn((
			CascadeChunk::unit_chunk().with_res_2(3),
			MeshDispatch::new(mesh_handle.clone()),
			Transform::from_translation(transform.translation + Vec3::new(0.003, 0.005, 0.004))
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
			Transform::from_translation(transform.translation + Vec3::new(0.0005, 0.0, 0.0005))
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
	) {
		let mut branch_builder = BranchBuilder::common_tree_builder();

		// anchor is on the ring of the trunk
		branch_builder.anchor = transform.translation + Vec3::new(0.0, 0.05, 0.005);

		// initial ray is sticking out to the side
		branch_builder.initial_ray = Vec3::new(0.0, 0.0, 1.0);

		// min segment length is 0.002
		branch_builder.min_segment_length = 0.02;

		// max segment length is 0.004
		branch_builder.max_segment_length = 0.04;

		// min radius is 0.002
		branch_builder.min_radius = 0.02;

		// max radius is 0.004
		branch_builder.max_radius = 0.04;

		let branch = branch_builder.build();

		println!("Builder: {:?}, Branch: {:?}", branch_builder, branch);

		// for now use the trunk segment
		let tree_segment = SimpleTrunkSegment::new(SegmentConfig::default());
		let mesh_handle = MeshHandle::new(tree_segment).with_handle_cache(self.tree_cache.clone());

		for segment in branch.segments() {
			let ray = segment.ray();
			let direction = ray.normalize();
			let length = ray.length();

			log::info!("Segment: {:?}, Direction: {:?}, Length: {:?}", segment, direction, length);

			let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

			let transform = Transform {
				translation: segment.start.position,
				rotation,
				scale: Vec3::new(segment.start.radius, length, segment.start.radius),
			};

			commands.spawn((
				cascade_chunk.clone(),
				MeshDispatch::new(mesh_handle.clone()),
				transform,
				MeshMaterial3d(material.0.clone()),
			));
		}
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

		self.spawn_branch(commands, cascade_chunk, transform, material);

		vec![]
	}
}
