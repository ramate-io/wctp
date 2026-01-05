use bevy::prelude::*;
use buildings::complex::{fillers::scratchpad::ScratchpadFiller, render::ComplexRenderer, Complex};
use chunk::cascade::CascadeChunk;
use engine::shaders::{leaf_material::LeafMaterial, outline::EdgeMaterial};
use render_item::{mesh::cache::handle::map::HandleMap, DispatchRenderItem};

#[derive(Resource, Clone)]
pub struct BuildingMaterial<M: Material>(pub Handle<M>);

pub fn setup_tree_edge_material(
	mut commands: Commands,
	mut materials: ResMut<Assets<EdgeMaterial>>,
) {
	let material_handle = materials.add(EdgeMaterial {
		// brownish color
		base_color: Vec4::new(0.89, 0.886, 0.604, 1.0),
	});

	commands.insert_resource(BuildingMaterial(material_handle));
}

pub fn building_playground<F: Material, P: Material>(
	mut commands: Commands,
	floor_material: Res<BuildingMaterial<F>>,
	partition_material: Res<BuildingMaterial<P>>,
) {
	log::info!("Spawning building playground");

	let partition_cache = HandleMap::<Partition>::new();

	commands.spawn((
		CascadeChunk::unit_center_chunk().with_res_2(3),
		DispatchRenderItem::new(grove),
		Transform::from_translation(Vec3::ZERO),
	));
}
