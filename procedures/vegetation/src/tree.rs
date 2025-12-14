pub mod canopy;
pub mod stalk;

use bevy::prelude::*;
pub use canopy::CanopySdf;
use chunk::cascade::CascadeChunk;
use render_item::RenderItem;
pub use stalk::{BranchSdf, RootSdf, TrunkSdf};

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
