use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
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
	log::info!("Spawning tree playground");

	commands.spawn((
		CascadeChunk::unit_center_chunk().with_res_2(7),
		DispatchRenderItem::new(TreeRenderItem::new()),
		Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(0.01, 0.01, 0.01)),
		MeshMaterial3d(material.0.clone()),
	));

	commands.spawn((
		CascadeChunk::unit_chunk().with_res_2(7),
		DispatchRenderItem::new(TreeRenderItem::new()),
		Transform::from_xyz(0.003, 0.005, 0.004)
			.with_scale(Vec3::new(0.005, 0.005, 0.005))
			.with_rotation(Quat::from_rotation_arc(Vec3::new(1.0, 1.0, 1.0).normalize(), Vec3::Y)),
		MeshMaterial3d(material.0.clone()),
	));

	commands.spawn((
		CascadeChunk::unit_chunk().with_res_2(7),
		DispatchRenderItem::new(TreeRenderItem::new()),
		Transform::from_xyz(0.0005, 0.0, 0.0005).with_scale(Vec3::new(0.009, 0.02, 0.009)),
		MeshMaterial3d(material.0.clone()),
	));
}
