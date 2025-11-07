use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::f32::consts::PI;

pub struct TerrainPlugin {
	pub seed: u32,
}

impl Plugin for TerrainPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(TerrainSeed(self.seed))
			.add_systems(Startup, (setup_camera, setup_terrain, setup_lighting))
			.add_systems(Update, camera_controller);
	}
}

#[derive(Resource)]
pub struct TerrainSeed(pub u32);

#[derive(Component)]
pub struct CameraController {
	pub speed: f32,
	pub sensitivity: f32,
	pub yaw: f32,
	pub pitch: f32,
}

fn setup_camera(mut commands: Commands) {
	commands.spawn((
		Camera3d::default(),
		Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
		Projection::Perspective(PerspectiveProjection::default()),
		CameraController {
			speed: 10.0,
			sensitivity: 0.001,
			yaw: -90.0_f32.to_radians(),
			pitch: -10.0_f32.to_radians(),
		},
	));
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

fn setup_terrain(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	seed: Res<TerrainSeed>,
) {
	let size = 100;
	let scale = 0.1;
	let height_scale = 5.0;

	// Create noise generator with seed
	let perlin = Perlin::new(seed.0);

	// Generate terrain mesh
	let mut vertices = Vec::new();
	let mut indices = Vec::new();
	let mut normals = Vec::new();
	let mut uvs = Vec::new();

	// Generate vertices
	for z in 0..=size {
		for x in 0..=size {
			let xf = x as f32;
			let zf = z as f32;

			// Generate height using multiple octaves of noise
			let mut height = 0.0;
			let mut amplitude = 1.0;
			let mut frequency = 0.05;
			let mut max_value = 0.0;

			for _ in 0..4 {
				let sample = perlin.get([xf as f64 * frequency, zf as f64 * frequency]) as f32;
				height += sample * amplitude;
				max_value += amplitude;
				amplitude *= 0.5;
				frequency *= 2.0;
			}

			height = (height / max_value) * height_scale;

			let y = height;
			vertices.push([xf - size as f32 / 2.0, y, zf - size as f32 / 2.0]);
			uvs.push([xf / size as f32, zf / size as f32]);
		}
	}

	// Generate indices for triangles
	for z in 0..size {
		for x in 0..size {
			let i = (z * (size + 1) + x) as u32;

			// First triangle
			indices.push(i);
			indices.push(i + size as u32 + 1);
			indices.push(i + 1);

			// Second triangle
			indices.push(i + 1);
			indices.push(i + size as u32 + 1);
			indices.push(i + size as u32 + 2);
		}
	}

	// Calculate normals
	normals.resize(vertices.len(), [0.0, 1.0, 0.0]);
	for i in (0..indices.len()).step_by(3) {
		let i0 = indices[i] as usize;
		let i1 = indices[i + 1] as usize;
		let i2 = indices[i + 2] as usize;

		let v0 = Vec3::from(vertices[i0]);
		let v1 = Vec3::from(vertices[i1]);
		let v2 = Vec3::from(vertices[i2]);

		let edge1 = v1 - v0;
		let edge2 = v2 - v0;
		let normal = edge1.cross(edge2).normalize();

		normals[i0] = (Vec3::from(normals[i0]) + normal).normalize().into();
		normals[i1] = (Vec3::from(normals[i1]) + normal).normalize().into();
		normals[i2] = (Vec3::from(normals[i2]) + normal).normalize().into();
	}

	// Create mesh
	let mut mesh = Mesh::new(
		bevy::render::mesh::PrimitiveTopology::TriangleList,
		bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
	);

	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
	mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
	mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

	// Create terrain material with color based on height
	let mesh_handle = meshes.add(mesh);
	let material_handle = materials.add(StandardMaterial {
		base_color: Color::srgb(0.2, 0.6, 0.3), // Green terrain
		metallic: 0.0,
		perceptual_roughness: 0.8,
		..default()
	});

	commands.spawn((
		Mesh3d(mesh_handle),
		MeshMaterial3d::<StandardMaterial>(material_handle),
		Transform::from_scale(Vec3::splat(scale)),
	));
}

fn camera_controller(
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
	time: Res<Time>,
	mut query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
) {
	let Ok((mut transform, mut controller)) = query.single_mut() else {
		return;
	};

	// Handle mouse look
	let mut mouse_delta = Vec2::ZERO;
	for event in mouse_motion.read() {
		mouse_delta += event.delta;
	}

	controller.yaw -= mouse_delta.x * controller.sensitivity;
	controller.pitch -= mouse_delta.y * controller.sensitivity;
	controller.pitch = controller.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

	// Update camera rotation
	let yaw_quat = Quat::from_axis_angle(Vec3::Y, controller.yaw);
	let pitch_quat = Quat::from_axis_angle(Vec3::X, controller.pitch);
	transform.rotation = yaw_quat * pitch_quat;

	// Handle movement
	let mut movement = Vec3::ZERO;
	let forward = transform.forward();
	let right = transform.right();

	if keyboard_input.pressed(KeyCode::KeyW) {
		movement += *forward;
	}
	if keyboard_input.pressed(KeyCode::KeyS) {
		movement -= *forward;
	}
	if keyboard_input.pressed(KeyCode::KeyA) {
		movement -= *right;
	}
	if keyboard_input.pressed(KeyCode::KeyD) {
		movement += *right;
	}
	if keyboard_input.pressed(KeyCode::Space) {
		movement += Vec3::Y;
	}
	if keyboard_input.pressed(KeyCode::ShiftLeft) {
		movement -= Vec3::Y;
	}

	if movement.length() > 0.0 {
		movement = movement.normalize() * controller.speed * time.delta_secs();
		transform.translation += movement;
	}
}
