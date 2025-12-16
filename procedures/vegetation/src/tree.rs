pub mod meshes;

use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use meshes::trunk::segment::{SegmentConfig, SimpleTrunkSegment};
use render_item::{
	mesh::{handle::MeshHandle, MeshDispatch},
	RenderItem,
};

#[derive(Component, Clone)]
pub struct TreeRenderItem {}

impl TreeRenderItem {
	pub fn new() -> Self {
		Self {}
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
		let mut entities = vec![];

		// Build tree segment dispatch
		let tree_segment = SimpleTrunkSegment::new(SegmentConfig::default());
		let mesh_handle = MeshHandle::new(tree_segment);
		let mesh_dispatch = MeshDispatch::new(mesh_handle);

		// spawn it
		entities
			.push(commands.spawn((cascade_chunk.clone(), mesh_dispatch, transform, material)).id());

		vec![]
	}
}
