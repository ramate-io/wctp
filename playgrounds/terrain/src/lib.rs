use bevy::prelude::*;
use std::f32::consts::PI;

mod camera;
mod chunk;
mod chunk_manager;
mod cpu;
mod geography;
mod gpu;
mod marching_cubes;
mod mesh_generator;
pub mod pipeline;
pub mod sdf;
pub mod shaders;
mod terrain;
mod ui;

pub use geography::FeatureRegistry;

pub use camera::CameraController;
pub use chunk::{ChunkConfig, ChunkCoord, LoadedChunks};
pub use terrain::TerrainConfig;

pub struct TerrainPlugin {
	pub seed: u32,
}

impl Plugin for TerrainPlugin {
	fn build(&self, app: &mut App) {
		// Set up geographic features
		let mut feature_registry = geography::FeatureRegistry::new();
		feature_registry
			.add_feature(Box::new(geography::canyons::CanyonFeature::new(self.seed, 1000)));

		let terrain_config = TerrainConfig::new(self.seed);
		let terrain_sdf = terrain::TerrainSdf { sdf: terrain::create_terrain_sdf(&terrain_config) };

		app.insert_resource(terrain_config)
			.insert_resource(terrain_sdf)
			.insert_resource(ClearColor(Color::hsla(201.0, 0.69, 0.62, 1.0)))
			.insert_resource(ChunkConfig::default())
			.insert_resource(LoadedChunks::default())
			.insert_resource(feature_registry)
			.insert_resource(mesh_generator::MeshGenerationMode::Cpu) // Default to GPU mode
			.add_systems(Startup, (camera::setup_camera, setup_lighting, ui::setup_debug_ui))
			.add_systems(
				Update,
				(
					camera::camera_controller,
					chunk_manager::manage_chunks,
					ui::update_coordinate_display,
				),
			);

		// Set up RenderApp systems for GPU pipeline initialization
		/*app.sub_app_mut(bevy::render::RenderApp).add_systems(
			bevy::render::RenderStartup,
			pipeline::proc::render_setup::queue_marching_cubes_pipelines,
		);

		app.sub_app_mut(bevy::render::RenderApp).add_systems(
			bevy::render::Render,
			pipeline::proc::render_setup::init_marching_cubes_pipelines
				.in_set(bevy::render::RenderSystems::Prepare),
		);*/

		// Extract pipelines from RenderApp to MainApp in Extract schedule
		app.add_systems(
			bevy::render::ExtractSchedule,
			pipeline::proc::render_setup::extract_pipelines_to_main_world,
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
