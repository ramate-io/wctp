pub mod meshes;

use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::RenderItem;

#[derive(Component, Clone)]
pub struct TreeRenderItem {}

impl RenderItem for TreeRenderItem {
	fn spawn_render_items<M: Material>(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<M>>,
	) -> Vec<Entity> {
		vec![]
	}
}
