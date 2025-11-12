use bevy::prelude::*;
use std::f32::consts::PI;

mod camera;
mod chunk;
mod chunk_manager;
mod geography;
mod marching_cubes;
pub mod sdf;
pub mod shaders;
mod terrain;
mod ui;
mod units;

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

		app.insert_resource(TerrainConfig::new(self.seed))
			.insert_resource(ClearColor(Color::hsla(201.0, 0.69, 0.62, 1.0)))
			.insert_resource(ChunkConfig::default())
			.insert_resource(LoadedChunks::default())
			.insert_resource(feature_registry)
			.add_systems(Startup, (camera::setup_camera, setup_lighting, ui::setup_debug_ui))
			.add_systems(
				Update,
				(
					camera::camera_controller,
					chunk_manager::manage_chunks,
					ui::update_coordinate_display,
					units::spawn_attached_cube,
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
