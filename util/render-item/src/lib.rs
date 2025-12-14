pub mod mesh;
pub mod sdf;

use bevy::prelude::*;
use chunk::cascade::CascadeChunk;

/// Used for logical items that can will spawn their constituens into the world.
pub trait RenderItem: Clone {
	fn spawn_render_items<M: Material>(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<M>>,
	) -> Vec<Entity>;
}

#[derive(Component)]
pub struct DispatchRenderItem<T: RenderItem> {
	item: T,
}

/// Spawns the render item to the world.
impl<T: RenderItem> DispatchRenderItem<T> {
	pub fn new(item: T) -> Self {
		Self { item }
	}

	pub fn spawn_render_items<M: Material>(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<M>>,
	) -> Vec<Entity> {
		self.item
			.spawn_render_items(commands, cascade_chunk, transform, meshes, materials)
	}
}

/// Handles the render items for a given cascade chunk, assigning them a material by type.
///
/// NOTE: this is not procedural contract for all produce all items of the type.
/// Rather, when a render item is dispatched, this begins the process of rendering said item.
pub fn render_items<T: RenderItem, M: Material>(
	commands: &mut Commands,
	dispatch_render_item: &DispatchRenderItem<T>,
	cascade_chunk: &CascadeChunk,
	transform: Transform,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<M>>,
) -> Vec<Entity> {
	dispatch_render_item.spawn_render_items(commands, cascade_chunk, transform, meshes, materials)
}
