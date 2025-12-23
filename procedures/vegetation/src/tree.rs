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

		let mut entities = vec![];

		// Build tree segment dispatch
		let tree_segment = SimpleTrunkSegment::new(SegmentConfig::default());
		let mesh_handle = MeshHandle::new(tree_segment).with_handle_cache(self.tree_cache.clone());
		let mesh_dispatch = MeshDispatch::new(mesh_handle);

		// spawn it
		entities
			.push(commands.spawn((cascade_chunk.clone(), mesh_dispatch, transform, material)).id());

		vec![]
	}
}
