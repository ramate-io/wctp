use bevy::prelude::*;
use std::f32::consts::PI;

mod camera;
mod checkerboard_material;
mod ground;
pub mod tree;
mod ui;

use engine::{
	manage_chunks, shaders::outline::EdgeMaterial, ChunkConfig, ChunkResolutionConfig,
	LoadedChunks, SdfResource,
};

pub use camera::CameraController;

pub use sdf;

pub struct ObjectsPlugin {
	pub seed: u32,
}

impl Plugin for ObjectsPlugin {
	fn build(&self, app: &mut App) {
		// Register EdgeMaterial plugin
		app.add_plugins(bevy::pbr::MaterialPlugin::<EdgeMaterial>::default());
		// Register CheckerboardMaterial plugin
		app.add_plugins(
			bevy::pbr::MaterialPlugin::<checkerboard_material::CheckerboardMaterial>::default(),
		);

		// Set up single tree SDF
		// let tree_chunk_config = ChunkConfig::<tree::TreeSdf> {
		// 	min_size: 0.01,
		// 	number_of_rings: 2,
		// 	grid_radius: 1,
		// 	grid_multiple_2: 4,
		// 	..Default::default()
		// };
		// let tree_resolution_config =
		// 	ChunkResolutionConfig::<tree::TreeSdf> { base_res_2: 7, ..Default::default() };
		// let tree_sdf = tree::create_tree_sdf();
		// let tree_sdf_resource = SdfResource::new(tree_sdf);

		// Set up simple segment SDF at origin
		// let segment_chunk_config = ChunkConfig::<tree::SegmentSdf> {
		// 	min_size: 0.01,
		// 	number_of_rings: 2,
		// 	grid_radius: 1,
		// 	grid_multiple_2: 2,
		// 	..Default::default()
		// };
		// let segment_resolution_config =
		// 	ChunkResolutionConfig::<tree::SegmentSdf> { base_res_2: 7, ..Default::default() };
		// let segment_sdf = tree::create_segment_sdf();
		// let segment_sdf_resource = SdfResource::new(segment_sdf);

		// Set up mesh-composition approach: three separate segments (base + 2 splits)
		let (base_segment, split_segment_1, split_segment_2) = tree::create_trunk_split_segments();

		// Configuration for base segment
		let base_chunk_config = ChunkConfig::<tree::BaseSegmentSdf> {
			min_size: 0.01,
			number_of_rings: 2,
			grid_radius: 1,
			grid_multiple_2: 2,
			..Default::default()
		};
		let base_resolution_config =
			ChunkResolutionConfig::<tree::BaseSegmentSdf> { base_res_2: 7, ..Default::default() };

		// Configuration for split segment 1
		let split1_chunk_config = ChunkConfig::<tree::SplitSegment1Sdf> {
			min_size: 0.01,
			number_of_rings: 2,
			grid_radius: 1,
			grid_multiple_2: 2,
			..Default::default()
		};
		let split1_resolution_config =
			ChunkResolutionConfig::<tree::SplitSegment1Sdf> { base_res_2: 7, ..Default::default() };

		// Configuration for split segment 2
		let split2_chunk_config = ChunkConfig::<tree::SplitSegment2Sdf> {
			min_size: 0.01,
			number_of_rings: 2,
			grid_radius: 1,
			grid_multiple_2: 2,
			..Default::default()
		};
		let split2_resolution_config =
			ChunkResolutionConfig::<tree::SplitSegment2Sdf> { base_res_2: 7, ..Default::default() };

		// Create SDF resources for each segment
		let base_sdf_resource = SdfResource::new(base_segment);
		let split1_sdf_resource = SdfResource::new(split_segment_1);
		let split2_sdf_resource = SdfResource::new(split_segment_2);

		app.insert_resource(ClearColor(Color::hsla(201.0, 0.69, 0.62, 1.0)))
			.insert_resource(ground::CheckerSize::default())
			// Base segment resources
			.insert_resource(base_chunk_config)
			.insert_resource(base_resolution_config)
			.insert_resource(base_sdf_resource)
			.insert_resource(LoadedChunks::default())
			// Split segment 1 resources
			.insert_resource(split1_chunk_config)
			.insert_resource(split1_resolution_config)
			.insert_resource(split1_sdf_resource)
			.insert_resource(LoadedChunks::default())
			// Split segment 2 resources
			.insert_resource(split2_chunk_config)
			.insert_resource(split2_resolution_config)
			.insert_resource(split2_sdf_resource)
			.insert_resource(LoadedChunks::default())
			.add_systems(
				Startup,
				(camera::setup_camera, setup_lighting, ground::setup_ground, ui::setup_debug_ui),
			)
			.add_systems(
				Update,
				(
					camera::camera_controller,
					ground::update_checker_size,
					manage_chunks::<tree::BaseSegmentSdf>,
					manage_chunks::<tree::SplitSegment1Sdf>,
					manage_chunks::<tree::SplitSegment2Sdf>,
					ui::update_coordinate_display,
				),
			);
	}
}

fn setup_lighting(mut commands: Commands) {
	// Ambient light - significantly increased to simulate global illumination
	// This provides base lighting for all surfaces, including back faces
	commands.insert_resource(AmbientLight {
		color: Color::WHITE,
		brightness: 2.0, // Much higher for better back-face illumination (simulates bounced light)
		affects_lightmapped_meshes: true,
	});

	// Main directional light (sun) - primary light source
	commands.spawn((
		DirectionalLight { illuminance: 10000.0, shadows_enabled: true, ..default() },
		Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
	));

	// Fill light from opposite direction - reduces harsh shadows
	commands.spawn((
		DirectionalLight {
			illuminance: 500.0,     // Increased fill light
			shadows_enabled: false, // No shadows for fill light
			..default()
		},
		Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, PI / 4.0, -PI / 4.0, 0.0)),
	));

	// Additional fill lights from sides for omnidirectional illumination
	// Left side
	commands.spawn((
		DirectionalLight { illuminance: 500.0, shadows_enabled: false, ..default() },
		Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, PI / 2.0, 0.0)),
	));

	// Right side
	commands.spawn((
		DirectionalLight { illuminance: 500.0, shadows_enabled: false, ..default() },
		Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, -PI / 2.0, 0.0)),
	));

	// Top-down fill light
	commands.spawn((
		DirectionalLight { illuminance: 500.0, shadows_enabled: false, ..default() },
		Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 2.0, 0.0, 0.0)),
	));
}
