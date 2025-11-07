use crate::chunk::{ChunkCoord, TerrainChunk};
use crate::geography::FeatureRegistry;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

/// Configuration for terrain generation
#[derive(Resource)]
pub struct TerrainConfig {
	pub seed: u32,
	pub base_resolution: usize, // Full resolution vertices per chunk side
	pub height_scale: f32,
}

impl TerrainConfig {
	pub fn new(seed: u32) -> Self {
		Self {
			seed,
			base_resolution: 128, // 50x50 vertices per chunk at full resolution
			height_scale: 5.0,
		}
	}

	/// Calculate resolution for a chunk based on Manhattan distance from camera
	/// Distance 0 (camera chunk) and 1 (immediate neighbors) always use full resolution
	/// Further chunks use decreasing resolution based on a power curve
	pub fn resolution_for_distance(&self, distance: i32) -> usize {
		// Camera chunk and immediate neighbors always full resolution
		if distance <= 2 {
			return self.base_resolution;
		}

		// For distance > 2, use exponential decay: resolution = base / 2^(distance-1)
		// This gives: distance 2 -> base/2, distance 3 -> base/4, distance 4 -> base/8, etc.
		let divisor = 2_i32.pow((distance - 1) as u32);
		let resolution = self.base_resolution / divisor as usize;

		// Ensure minimum resolution of at least 4 vertices (2x2 grid minimum)
		resolution.max(4)
	}
}

/// Generate a terrain mesh for a specific chunk
pub fn generate_chunk_mesh(
	chunk_coord: &ChunkCoord,
	chunk_size: f32,
	resolution: usize,
	config: &TerrainConfig,
	perlin: &Perlin,
	feature_registry: Option<&FeatureRegistry>,
) -> Mesh {
	let mut vertices = Vec::new();
	let mut indices = Vec::new();
	let mut normals = Vec::new();
	let mut uvs = Vec::new();

	// Calculate world offset for this chunk
	// Use unwrapped coordinates for noise generation to ensure seamless wrapping
	let chunk_origin_unwrapped = chunk_coord.to_unwrapped_world_pos(chunk_size);
	let step = chunk_size / resolution as f32;

	// Generate vertices
	for z in 0..=resolution {
		for x in 0..=resolution {
			let xf = x as f32;
			let zf = z as f32;

			// World position (unwrapped for seamless noise generation)
			let world_x = chunk_origin_unwrapped.x + xf * step;
			let world_z = chunk_origin_unwrapped.z + zf * step;

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

			// Apply geographic features (canyons, etc.)
			if let Some(registry) = feature_registry {
				height = registry.apply_features(world_x, world_z, height, config);
			}

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
/// wrapped_coord: Used for indexing/uniqueness (wrapped to world bounds)
/// unwrapped_coord: Used for world position (actual position in space)
pub fn spawn_chunk(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
	wrapped_coord: ChunkCoord,
	unwrapped_coord: ChunkCoord,
	chunk_size: f32,
	_world_size_chunks: i32,
	resolution: usize,
	config: &TerrainConfig,
	perlin: &Perlin,
	feature_registry: Option<&FeatureRegistry>,
) -> Entity {
	// Use unwrapped coordinate for mesh generation to ensure seamless terrain
	let mesh = generate_chunk_mesh(&unwrapped_coord, chunk_size, resolution, config, perlin, feature_registry);
	let mesh_handle = meshes.add(mesh);

	// Make the origin chunk (0, 0) reddish for easy verification
	let is_origin_chunk = wrapped_coord.x == 0 && wrapped_coord.z == 0;
	let base_color = if is_origin_chunk {
		Color::srgb(0.8, 0.2, 0.2) // Reddish for origin chunk
	} else {
		Color::srgb(0.2, 0.6, 0.3) // Green terrain
	};

	let material_handle = materials.add(StandardMaterial {
		base_color,
		metallic: 0.0,
		perceptual_roughness: 0.8,
		..default()
	});

	// Use unwrapped coordinate for world position (spawn at actual location)
	let world_pos = unwrapped_coord.to_unwrapped_world_pos(chunk_size);

	let entity = commands
		.spawn((
			TerrainChunk { coord: wrapped_coord, resolution }, // Store wrapped for indexing
			Mesh3d(mesh_handle),
			MeshMaterial3d::<StandardMaterial>(material_handle),
			Transform::from_translation(world_pos),
		))
		.id();

	log::debug!(
		"Spawned chunk wrapped=({}, {}) unwrapped=({}, {}) at world position {:?} with resolution {}",
		wrapped_coord.x,
		wrapped_coord.z,
		unwrapped_coord.x,
		unwrapped_coord.z,
		world_pos,
		resolution
	);

	entity
}
