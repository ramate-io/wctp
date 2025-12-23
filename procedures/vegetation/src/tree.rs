pub mod meshes;

use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use meshes::trunk::segment::{SegmentConfig, SimpleTrunkSegment};
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
}

impl RenderItem for TreeRenderItem {
	fn spawn_render_items<M: Material>(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		material: MeshMaterial3d<M>,
	) -> Vec<Entity> {
		log::info!("Spawning tree render items");

		self.spawn_trunk(commands, cascade_chunk, transform, material);

		vec![]
	}
}
