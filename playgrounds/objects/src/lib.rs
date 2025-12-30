use bevy::prelude::*;
use std::f32::consts::PI;

mod camera;
mod checkerboard_material;
mod ground;
pub mod tree;
mod ui;

use engine::shaders::{leaf_material::LeafMaterial, outline::EdgeMaterial};
use vegetation_sdf::tree::{meshes::trunk::segment::SimpleTrunkSegment, TreeRenderItem};

use render_item::{
	mesh::{fetch_meshes, handle::MeshHandle},
	render_items,
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
		app.add_plugins(bevy::pbr::MaterialPlugin::<LeafMaterial>::default());
		// Register CheckerboardMaterial plugin
		app.add_plugins(
			bevy::pbr::MaterialPlugin::<checkerboard_material::CheckerboardMaterial>::default(),
		);

		app.insert_resource(ClearColor(Color::hsla(201.0, 0.69, 0.62, 1.0)))
			.insert_resource(ground::CheckerSize::default())
			.add_systems(
				Startup,
				(
					camera::setup_camera,
					setup_lighting,
					ground::setup_ground,
					ui::setup_debug_ui,
					tree::setup_tree_edge_material,
					setup_leaf_ball,
				),
			)
			.add_systems(
				Update,
				(
					camera::camera_controller,
					ground::update_checker_size,
					ui::update_coordinate_display,
					render_items::<TreeRenderItem, EdgeMaterial>,
					fetch_meshes::<MeshHandle<SimpleTrunkSegment>, EdgeMaterial>,
					tree::tree_playground::<EdgeMaterial>
						.run_if(resource_exists::<tree::TreeMaterial<EdgeMaterial>>)
						.run_if(run_once),
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

fn setup_leaf_ball(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<LeafMaterial>>,
) {
	// Create leaf material with green color
	let leaf_material = materials.add(LeafMaterial {
		base_color: Vec4::new(0.2, 0.8, 0.3, 1.0), // Green color
	});

	// Create a disc mesh (small size for leaves)
	let leaf_size = 0.01; // Size in kilometers (radius)
	let disc_mesh = meshes.add(Circle::new(leaf_size));

	// Spawn 3-8 discs that intersect at one point and fan out like a star
	let num_discs = 8; // Number of discs (3-8, adjust as needed)
	let center = Vec3::new(0.0, 0.03, 0.0); // All planes intersect at origin

	// Use Fibonacci sphere algorithm for even distribution of directions
	let golden_angle = PI * (3.0 - (5.0_f32).sqrt()); // Golden angle in radians

	for i in 0..num_discs {
		// Calculate direction using Fibonacci sphere for even distribution
		let theta = golden_angle * i as f32;
		let y = 1.0 - (2.0 * i as f32) / (num_discs as f32 - 1.0); // y goes from 1 to -1
		let radius_at_y = (1.0 - y * y).sqrt(); // Radius at this y level

		// Calculate direction vector (normal of the disc)
		let x = radius_at_y * theta.cos();
		let z = radius_at_y * theta.sin();
		let direction = Vec3::new(x, y, z).normalize();

		// All discs are at the same point (slightly offset to avoid z-fighting)
		let position = center + direction * 0.00001; // Tiny offset

		// Create rotation so disc normal points in the direction
		// Disc's default normal is Vec3::Z, so rotate Z to direction
		let rotation = if direction.abs_diff_eq(Vec3::Z, 1e-4) {
			Quat::IDENTITY
		} else {
			Quat::from_rotation_arc(Vec3::Z, direction)
		};

		commands.spawn((
			Mesh3d(disc_mesh.clone()),
			MeshMaterial3d(leaf_material.clone()),
			Transform { translation: position, rotation, scale: Vec3::ONE },
		));
	}
}
