use crate::chunk::{ChunkCoord, TerrainChunk};
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

/// Configuration for terrain generation
#[derive(Resource)]
pub struct TerrainConfig {
	pub seed: u32,
	pub resolution: usize, // vertices per chunk side
	pub height_scale: f32,
}

impl TerrainConfig {
	pub fn new(seed: u32) -> Self {
		Self {
			seed,
			resolution: 50, // 50x50 vertices per chunk
			height_scale: 5.0,
		}
	}
}

/// Generate a terrain mesh for a specific chunk
pub fn generate_chunk_mesh(
	chunk_coord: &ChunkCoord,
	chunk_size: f32,
	config: &TerrainConfig,
	perlin: &Perlin,
) -> Mesh {
	let resolution = config.resolution;
	let mut vertices = Vec::new();
	let mut indices = Vec::new();
	let mut normals = Vec::new();
	let mut uvs = Vec::new();

	// Calculate world offset for this chunk
	let chunk_origin = chunk_coord.to_world_origin(chunk_size);
	let step = chunk_size / resolution as f32;

	// Generate vertices
	for z in 0..=resolution {
		for x in 0..=resolution {
			let xf = x as f32;
			let zf = z as f32;

			// World position
			let world_x = chunk_origin.x + xf * step;
			let world_z = chunk_origin.z + zf * step;

			// Generate height using multiple octaves of noise
			let mut height = 0.0;
			let mut amplitude = 1.0;
			let mut frequency = 0.05;
			let mut max_value = 0.0;

			for _ in 0..4 {
				let sample =
					perlin.get([world_x as f64 * frequency, world_z as f64 * frequency]) as f32;
				height += sample * amplitude;
				max_value += amplitude;
				amplitude *= 0.5;
				frequency *= 2.0;
			}

			height = (height / max_value) * config.height_scale;

			// Local position relative to chunk origin
			let local_x = xf * step;
			let local_z = zf * step;
			vertices.push([local_x, height, local_z]);
			uvs.push([xf / resolution as f32, zf / resolution as f32]);
		}
	}

	// Generate indices for triangles
	for z in 0..resolution {
		for x in 0..resolution {
			let i = (z * (resolution + 1) + x) as u32;

			// First triangle
			indices.push(i);
			indices.push(i + resolution as u32 + 1);
			indices.push(i + 1);

			// Second triangle
			indices.push(i + 1);
			indices.push(i + resolution as u32 + 1);
			indices.push(i + resolution as u32 + 2);
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

	mesh
}

/// Spawn a terrain chunk entity
pub fn spawn_chunk(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
	chunk_coord: ChunkCoord,
	chunk_size: f32,
	config: &TerrainConfig,
	perlin: &Perlin,
) -> Entity {
	let mesh = generate_chunk_mesh(&chunk_coord, chunk_size, config, perlin);
	let mesh_handle = meshes.add(mesh);

	let material_handle = materials.add(StandardMaterial {
		base_color: Color::srgb(0.2, 0.6, 0.3), // Green terrain
		metallic: 0.0,
		perceptual_roughness: 0.8,
		..default()
	});

	let world_pos = chunk_coord.to_world_origin(chunk_size);

	let entity = commands
		.spawn((
			TerrainChunk { coord: chunk_coord },
			Mesh3d(mesh_handle),
			MeshMaterial3d::<StandardMaterial>(material_handle),
			Transform::from_translation(world_pos),
		))
		.id();

	log::debug!(
		"Spawned chunk at ({}, {}) at world position {:?}",
		chunk_coord.x,
		chunk_coord.z,
		world_pos
	);

	entity
}
