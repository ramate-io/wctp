use crate::chunk::{ChunkCoord, TerrainChunk};
use crate::geography::FeatureRegistry;
use crate::marching_cubes::EDGE_VERTEX_INDICES;
use crate::sdf::{PerlinTerrainSdf, Sdf};
use bevy::prelude::*;

/// Configuration for terrain generation
#[derive(Resource, Clone)]
pub struct TerrainConfig {
	pub seed: u32,
	pub base_resolution: usize, // Full resolution vertices per chunk side
	pub height_scale: f32,
	pub use_volumetric: bool, // If true, use marching cubes; if false, use heightfield
}

impl TerrainConfig {
	pub fn new(seed: u32) -> Self {
		Self {
			seed,
			base_resolution: 128, // 128x128 vertices per chunk at full resolution
			height_scale: 5.0,
			use_volumetric: true, // Default to volumetric for true 3D terrain
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

/// Generate a terrain mesh for a specific chunk by sampling an SDF
/// Supports both heightfield (fast, no caves) and volumetric (marching cubes, supports caves)
pub fn generate_chunk_mesh(
	chunk_coord: &ChunkCoord,
	chunk_size: f32,
	resolution: usize,
	config: &TerrainConfig,
	feature_registry: Option<&FeatureRegistry>,
) -> Mesh {
	if config.use_volumetric {
		generate_chunk_mesh_volumetric(
			chunk_coord,
			chunk_size,
			resolution,
			config,
			feature_registry,
		)
	} else {
		generate_chunk_mesh_heightfield(
			chunk_coord,
			chunk_size,
			resolution,
			config,
			feature_registry,
		)
	}
}

/// Generate mesh using heightfield approach (fast, but no caves/overhangs)
fn generate_chunk_mesh_heightfield(
	chunk_coord: &ChunkCoord,
	chunk_size: f32,
	resolution: usize,
	config: &TerrainConfig,
	feature_registry: Option<&FeatureRegistry>,
) -> Mesh {
	let mut vertices = Vec::new();
	let mut indices = Vec::new();
	let mut normals = Vec::new();
	let mut uvs = Vec::new();

	// Create SDF for terrain generation
	let sdf = PerlinTerrainSdf::new(config.seed, config.clone(), feature_registry);

	// Calculate world offset for this chunk
	let chunk_origin_unwrapped = chunk_coord.to_unwrapped_world_pos(chunk_size);
	let step = chunk_size / resolution as f32;

	// Generate vertices by sampling the SDF
	for z in 0..=resolution {
		for x in 0..=resolution {
			let xf = x as f32;
			let zf = z as f32;

			// World position (unwrapped for seamless noise generation)
			let world_x = chunk_origin_unwrapped.x + xf * step;
			let world_z = chunk_origin_unwrapped.z + zf * step;

			// Sample the SDF to find surface height
			let height = find_surface_height(&sdf, world_x, world_z, config.height_scale);

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

pub fn generate_chunk_mesh_volumetric(
	chunk_coord: &ChunkCoord,
	chunk_size: f32,
	res: usize,
	config: &TerrainConfig,
	feature_registry: Option<&FeatureRegistry>,
) -> Mesh {
	let sdf = PerlinTerrainSdf::new(config.seed, config.clone(), feature_registry);

	// ---------- grid setup ---------------------------------------------------
	let cube_size = chunk_size / res as f32;
	let chunk_origin = chunk_coord.to_unwrapped_world_pos(chunk_size);

	// Vertical sampling range in world Y
	let y_min = -config.height_scale * 2.0;
	let y_max = config.height_scale * 2.0;
	let y_range = y_max - y_min;

	// Number of cells vertically to roughly match cube_size
	let y_cells = (y_range / cube_size).ceil().max(1.0) as usize;

	// Grid resolution (sample points); cubes are (n-1) in each axis
	let nx = res + 1;
	let nz = res + 1;
	let ny = y_cells + 1;

	// Helper: linear index with X fastest, then Z, then Y (consistent)
	let idx = |x: usize, y: usize, z: usize| -> usize { (y * nz + z) * nx + x };

	// Scalar field samples
	let mut grid = vec![0.0f32; nx * ny * nz];

	// ---------- sample SDF in world space -----------------------------------
	for y in 0..ny {
		let wy = y_min + y as f32 * cube_size;
		for z in 0..nz {
			let wz = chunk_origin.z + z as f32 * cube_size;
			for x in 0..nx {
				let wx = chunk_origin.x + x as f32 * cube_size;
				grid[idx(x, y, z)] = sdf.distance(Vec3::new(wx, wy, wz));
			}
		}
	}

	// ---------- Marching Cubes ----------------------------------------------
	use crate::marching_cubes::{get_cube_index, interpolate_vertex, TRIANGULATIONS};

	// Output buffers
	let mut vertices: Vec<[f32; 3]> = Vec::new();
	let mut indices: Vec<u32> = Vec::new();

	// Number of cubes along each axis
	let cx = nx - 1;
	let cy = ny - 1;
	let cz = nz - 1;

	// Iterate over cubes
	for y in 0..cy {
		for z in 0..cz {
			for x in 0..cx {
				// Corner scalar values (standard MC corner ordering assumed by your helpers)
				let corners = [
					grid[idx(x, y, z)],             // 0 (0,0,0)
					grid[idx(x + 1, y, z)],         // 1 (1,0,0)
					grid[idx(x + 1, y, z + 1)],     // 2 (1,0,1)  <-- FIX
					grid[idx(x, y, z + 1)],         // 3 (0,0,1)  <-- FIX
					grid[idx(x, y + 1, z)],         // 4 (0,1,0)
					grid[idx(x + 1, y + 1, z)],     // 5 (1,1,0)
					grid[idx(x + 1, y + 1, z + 1)], // 6 (1,1,1)  <-- FIX
					grid[idx(x, y + 1, z + 1)],     // 7 (0,1,1)  <-- FIX
				];

				let cube_index = get_cube_index(corners);
				if cube_index == 0 || cube_index == 255 {
					continue; // fully inside or outside
				}

				// Local-space cube origin (NOTE: local X/Z, absolute Y)
				//  - X/Z are local to the chunk; your entity transform can place the chunk at chunk_origin
				//  - Y is absolute because we sampled the field in absolute Y (y_min baseline)
				let cube_pos_local = Vec3::new(
					x as f32 * cube_size,
					y_min + y as f32 * cube_size,
					z as f32 * cube_size,
				);

				// Per-cube edge vertex cache (12 edges)
				let mut edge_vert: [Option<u32>; 12] = [None; 12];

				let tri = &TRIANGULATIONS[cube_index];
				let mut i = 0;
				while i + 2 < tri.len() {
					let e0 = tri[i];
					if e0 < 0 {
						break;
					}
					let e1 = tri[i + 1];
					if e1 < 0 {
						break;
					}
					let e2 = tri[i + 2];
					if e2 < 0 {
						break;
					}

					let mut get_vert = |edge: usize| -> u32 {
						if let Some(v) = edge_vert[edge] {
							return v;
						}
						let pos_local = edge_midpoint(edge, cube_pos_local, cube_size);
						let v_index = vertices.len() as u32;
						vertices.push([pos_local.x, pos_local.y, pos_local.z]);
						edge_vert[edge] = Some(v_index);
						v_index
					};

					let v0 = get_vert(e0 as usize);
					let v1 = get_vert(e1 as usize);
					let v2 = get_vert(e2 as usize);

					indices.extend_from_slice(&[v0, v1, v2]);
					i += 3;
				}
			}
		}
	}

	// ---------- Normals & UVs -----------------------------------------------
	// Normals: sample SDF gradient in WORLD space for shading correctness.
	// Convert local X/Z back to world by adding chunk_origin; Y is already absolute.
	let normals: Vec<[f32; 3]> = vertices
		.iter()
		.map(|v| {
			let world = Vec3::new(v[0] + chunk_origin.x, v[1], v[2] + chunk_origin.z);
			calculate_sdf_normal(&sdf, world).into()
		})
		.collect();

	// Simple tiled UVs (local X/Z across the chunk)
	let uvs: Vec<[f32; 2]> =
		vertices.iter().map(|v| [v[0] / chunk_size, v[2] / chunk_size]).collect();

	// ---------- Mesh ---------------------------------------------------------
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

fn edge_midpoint(edge: usize, cube_pos_local: Vec3, cube_size: f32) -> Vec3 {
	let (a, b) = EDGE_VERTEX_INDICES[edge];
	let corner_pos = [
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(1.0, 0.0, 0.0),
		Vec3::new(1.0, 0.0, 1.0),
		Vec3::new(0.0, 0.0, 1.0),
		Vec3::new(0.0, 1.0, 0.0),
		Vec3::new(1.0, 1.0, 0.0),
		Vec3::new(1.0, 1.0, 1.0),
		Vec3::new(0.0, 1.0, 1.0),
	];
	let p1 = corner_pos[a];
	let p2 = corner_pos[b];
	// midpoint in local cube space
	let local_mid = (p1 + p2) * 0.5;
	cube_pos_local + local_mid * cube_size
}

/// Calculate normal from SDF gradient
fn calculate_sdf_normal(sdf: &impl Sdf, p: Vec3) -> Vec3 {
	// Use smaller epsilon to avoid exaggerating streaks
	let epsilon = 0.0005;
	let dx = sdf.distance(Vec3::new(p.x + epsilon, p.y, p.z))
		- sdf.distance(Vec3::new(p.x - epsilon, p.y, p.z));
	let dy = sdf.distance(Vec3::new(p.x, p.y + epsilon, p.z))
		- sdf.distance(Vec3::new(p.x, p.y - epsilon, p.z));
	let dz = sdf.distance(Vec3::new(p.x, p.y, p.z + epsilon))
		- sdf.distance(Vec3::new(p.x, p.y, p.z - epsilon));

	let grad = Vec3::new(dx, dy, dz);
	let len = grad.length();
	if len > 0.0001 {
		grad / len
	} else {
		Vec3::Y // Fallback to up if gradient is too small
	}
}

/// Find the surface height by sampling the SDF
/// Uses binary search along Y axis to find where distance crosses zero
fn find_surface_height(sdf: &impl Sdf, world_x: f32, world_z: f32, max_height: f32) -> f32 {
	// Search range: from well below ground to well above max terrain height
	let y_min = -max_height * 2.0;
	let y_max = max_height * 2.0;
	let epsilon = 0.01; // Precision threshold

	// Binary search for zero crossing
	let mut low = y_min;
	let mut high = y_max;

	for _ in 0..32 {
		// Limit iterations to prevent infinite loops
		let mid = (low + high) * 0.5;
		let distance = sdf.distance(Vec3::new(world_x, mid, world_z));

		if distance.abs() < epsilon {
			return mid;
		}

		if distance > 0.0 {
			// Above surface, search lower
			high = mid;
		} else {
			// Below surface, search higher
			low = mid;
		}
	}

	// Fallback: if binary search didn't converge, use the midpoint
	(low + high) * 0.5
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
	feature_registry: Option<&FeatureRegistry>,
) -> Entity {
	// Use unwrapped coordinate for mesh generation to ensure seamless terrain
	let mesh =
		generate_chunk_mesh(&unwrapped_coord, chunk_size, resolution, config, feature_registry);
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
		perceptual_roughness: 0.7, // Less rough for more light reflection/bounce
		..default()
	});

	// Use unwrapped coordinate for world position (spawn at actual location)
	// Note: mesh vertices are in local space relative to chunk origin
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
