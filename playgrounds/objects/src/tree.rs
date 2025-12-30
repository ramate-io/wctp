use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use engine::shaders::{leaf_material::LeafMaterial, outline::EdgeMaterial};
use render_item::{mesh::cache::handle::map::HandleMap, DispatchRenderItem};
use vegetation_sdf::tree::{meshes::trunk::segment::SimpleTrunkSegment, TreeRenderItem};

#[derive(Resource, Clone)]
pub struct TreeMaterial<M: Material>(pub Handle<M>);

pub fn setup_tree_edge_material(
	mut commands: Commands,
	mut materials: ResMut<Assets<EdgeMaterial>>,
	mut leaf_materials: ResMut<Assets<LeafMaterial>>,
) {
	let material_handle = materials.add(EdgeMaterial {
		// brownish color
		base_color: Vec4::new(0.89, 0.886, 0.604, 1.0),
	});

	let leaf_material_handle =
		leaf_materials.add(LeafMaterial { base_color: Vec4::new(0.89, 0.886, 0.604, 1.0) });

	commands.insert_resource(TreeMaterial(material_handle));
	commands.insert_resource(TreeMaterial(leaf_material_handle));
}

pub fn tree_playground<M: Material>(mut commands: Commands, material: Res<TreeMaterial<M>>) {
	log::info!("Spawning tree playground");

	let tree_cache = HandleMap::<SimpleTrunkSegment>::new();

	// grid out some trees
	const N: i32 = 1;
	for x in -N..=N {
		for z in -N..=N {
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
		Transform::from_translation(origin),
		MeshMaterial3d(material.0.clone()),
	));
}
