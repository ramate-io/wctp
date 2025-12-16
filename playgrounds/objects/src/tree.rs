use bevy::prelude::*;
use engine::shaders::outline::EdgeMaterial;
use render_item::DispatchRenderItem;
use vegetation_sdf::tree::TreeRenderItem;

#[derive(Resource, Clone)]
pub struct TreeMaterial<M: Material>(pub Handle<M>);

pub fn setup_tree_edge_material(
	mut commands: Commands,
	mut materials: ResMut<Assets<EdgeMaterial>>,
) {
	let material_handle = materials.add(EdgeMaterial {
		// brownish color
		base_color: Vec4::new(0.89, 0.886, 0.604, 1.0),
	});

	commands.insert_resource(TreeMaterial(material_handle));
}

pub fn tree_playground<M: Material>(mut commands: Commands, material: Res<TreeMaterial<M>>) {
	commands.spawn((
		DispatchRenderItem::new(TreeRenderItem::new()),
		Transform::from_xyz(0.0, 0.0, 0.0),
		MeshMaterial3d(material.0.clone()),
	));
}
