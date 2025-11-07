use bevy::prelude::*;
use std::f32::consts::PI;

mod camera;
mod chunk;
mod chunk_manager;
mod terrain;
mod ui;

pub use camera::CameraController;
pub use chunk::{ChunkConfig, ChunkCoord, LoadedChunks};
pub use terrain::TerrainConfig;

pub struct TerrainPlugin {
	pub seed: u32,
}

impl Plugin for TerrainPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(TerrainConfig::new(self.seed))
			.insert_resource(ChunkConfig::default())
			.insert_resource(LoadedChunks::default())
			.add_systems(Startup, (camera::setup_camera, setup_lighting, ui::setup_debug_ui))
			.add_systems(
				Update,
				(
					camera::camera_controller,
					chunk_manager::manage_chunks,
					ui::update_coordinate_display,
				),
			);
	}
}

fn setup_lighting(mut commands: Commands) {
	// Ambient light
	commands.insert_resource(AmbientLight {
		color: Color::WHITE,
		brightness: 0.3,
		affects_lightmapped_meshes: true,
	});

	// Directional light (sun)
	commands.spawn((
		DirectionalLight { illuminance: 10000.0, shadows_enabled: true, ..default() },
		Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
	));
}
