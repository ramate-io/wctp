use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use engine::shaders::outline::EdgeMaterial;
use render_item::{mesh::cache::handle::map::HandleMap, DispatchRenderItem};
use vegetation_sdf::tree::{meshes::trunk::segment::SimpleTrunkSegment, TreeRenderItem};

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

	let tree_cache = HandleMap::<SimpleTrunkSegment>::new();

	// grid out some trees
	for x in -100..=100 {
		for z in -100..=100 {
			tree(
				&mut commands,
				Vec3::new(x as f32 * 0.02, 0.0, z as f32 * 0.02),
				&material,
				tree_cache.clone(),
			);
		}
	}
}

pub fn tree<M: Material>(
	commands: &mut Commands,
	origin: Vec3,
	material: &Res<TreeMaterial<M>>,
	tree_cache: HandleMap<SimpleTrunkSegment>,
) {
	commands.spawn((
		CascadeChunk::unit_center_chunk().with_res_2(3),
		DispatchRenderItem::new(TreeRenderItem::new().with_tree_cache(tree_cache.clone())),
		Transform::from_translation(origin + Vec3::new(0.0, 0.0, 0.0))
			.with_scale(Vec3::new(0.01, 0.01, 0.01)),
		MeshMaterial3d(material.0.clone()),
	));

	commands.spawn((
		CascadeChunk::unit_chunk().with_res_2(3),
		DispatchRenderItem::new(TreeRenderItem::new().with_tree_cache(tree_cache.clone())),
		Transform::from_translation(origin + Vec3::new(0.003, 0.005, 0.004))
			.with_scale(Vec3::new(0.005, 0.005, 0.005))
			.with_rotation(Quat::from_rotation_arc(Vec3::new(1.0, 1.0, 1.0).normalize(), Vec3::Y)),
		MeshMaterial3d(material.0.clone()),
	));

	commands.spawn((
		CascadeChunk::unit_chunk().with_res_2(3),
		DispatchRenderItem::new(TreeRenderItem::new().with_tree_cache(tree_cache.clone())),
		Transform::from_translation(origin + Vec3::new(0.0005, 0.0, 0.0005))
			.with_scale(Vec3::new(0.009, 0.02, 0.009)),
		MeshMaterial3d(material.0.clone()),
	));
}
