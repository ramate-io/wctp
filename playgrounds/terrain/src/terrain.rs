use crate::chunk::{ChunkCoord, TerrainChunk};
// use crate::geography::FeatureRegistry;
use crate::sdf::{
	region::{
		affine::RegionAffineModulation, rounding::RegionRoundingModulation, branching::BranchingPlan, CircleRegion, RectRegion,
		Region2D, RegionNoise,
		grading::RegionGradingModulation,
	},
	Difference, Ellipse3d, PerlinTerrainSdf, Sdf, TubeSdf,
};
use bevy::prelude::*;
use noise::Perlin;

/// Resource containing the terrain SDF for runtime queries
#[derive(Resource)]
pub struct TerrainSdf {
	pub sdf: Box<dyn Sdf>,
}

/// Create the terrain SDF with all modulations
pub fn create_terrain_sdf(config: &TerrainConfig) -> Box<dyn Sdf> {
	// Create base terrain SDF
	let mut sdf = PerlinTerrainSdf::new(config.seed, config.clone());

	let big_valley_sdf = RegionAffineModulation::new(
		Region2D::Rect(RectRegion {
			center: Vec2::new(20.0, 20.0),
			half_extents: Vec2::new(90.0, 90.0),
			round: 2.0,
		}),
		0.5,
		0.0,
		10.0,
		10.0,
	)
	.with_noise(RegionNoise { noise: Perlin::new(config.seed), frequency: 0.2, amplitude: 2.0 });

	let intersecting_big_valley_sdf = RegionAffineModulation::new(
		Region2D::Circle(CircleRegion { center: Vec2::new(10.0, 70.0), radius: 80.0 }),
		0.5,
		-1.7,
		10.0,
		10.0,
	)
	.with_noise(RegionNoise { noise: Perlin::new(config.seed), frequency: 0.2, amplitude: 2.0 });

	sdf.add_elevation_modulation(Box::new(intersecting_big_valley_sdf));

	// branching regions
	let branch_plan = BranchingPlan::new(big_valley_sdf, Perlin::new(config.seed), 3, 2);

	let modulations = branch_plan.generate_regions();

	for modulation in modulations {
		sdf.add_elevation_modulation(Box::new(modulation));
	}

	let road_sdf = RegionRoundingModulation::new(
		Region2D::Rect(RectRegion { center: Vec2::new(0.0, 0.0), half_extents: Vec2::new(80.0, 1.0), round: 0.1 }),
		0.01,
		None,
		0.4,
		0.2,
	);

	sdf.add_elevation_modulation(Box::new(road_sdf));

	let start_point = Vec2::new(0.0, 20.0);
	let start_elevation = sdf.height_at_with_all_modulations(start_point.x, start_point.y);
	let end_point = Vec2::new(40.0, 20.0);
	let end_elevation = sdf.height_at_with_all_modulations(end_point.x, end_point.y);

	let graded_road = RegionGradingModulation::new(
		Region2D::Rect(RectRegion { center: Vec2::new(20.0, 20.0), half_extents: Vec2::new(20.0, 1.0), round: 0.01 }),
		start_point,
		start_elevation,
		end_point,
		end_elevation,
		None,
		0.4,
		0.1,
	);

	sdf.add_elevation_modulation(Box::new(graded_road));

	// Create a large vertical tube to bore a hole through the terrain
	// Position it near the origin, going from well below ground to well above
	let tube_start = Vec3::new(-30.0, -1.0, -30.0); // Start deep below
	let tube_end = Vec3::new(-50.0, 4.0, -50.0); // End high above

	// Create a circular cross-section (ellipse with equal radii)
	// Make it quite large - 15 unit radius
	let tube_center = Vec3::new(20.0, 0.0, 20.0);
	let tube_axis = (tube_end - tube_start).normalize();

	// Build orthonormal basis perpendicular to tube axis
	let right = if tube_axis.x.abs() > tube_axis.z.abs() {
		Vec3::new(-tube_axis.y, tube_axis.x, 0.0).normalize()
	} else {
		Vec3::new(0.0, -tube_axis.z, tube_axis.y).normalize()
	};
	let up = tube_axis.cross(right).normalize();

	let tube_ellipse = Ellipse3d {
		center: tube_center,
		axes: [right, up],
		radii: Vec2::new(2.0, 2.0), // Large circular cross-section
	};

	let tube_sdf = TubeSdf::new(tube_start, tube_end, tube_ellipse)
		.with_noise(Perlin::new(config.seed))
		.with_noise_factor(0.4);

	// Use Difference to bore the hole (subtract tube from terrain)
	Box::new(Difference::new(sdf, tube_sdf))
}

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
		((self.base_resolution as f32) - (distance as f32 * 30.0)).max(32.0) as usize
	}
}

/// Generate a terrain mesh for a specific chunk by sampling an SDF
/// Supports both heightfield (fast, no caves) and volumetric (marching cubes, supports caves)
pub fn generate_chunk_mesh(
	chunk_coord: &ChunkCoord,
	chunk_size: f32,
	resolution: usize,
	config: &TerrainConfig,
	// feature_registry: Option<&FeatureRegistry>,
) -> Mesh {
	generate_chunk_mesh_volumetric(chunk_coord, chunk_size, resolution, config)
}

pub fn generate_chunk_mesh_volumetric(
	chunk_coord: &ChunkCoord,
	chunk_size: f32,
	res: usize,
	config: &TerrainConfig,
	// feature_registry: Option<&FeatureRegistry>,
) -> Mesh {
	// Use the same SDF creation logic
	let sdf = create_terrain_sdf(config);

	// ---------- grid setup ---------------------------------------------------
	let cube_size = chunk_size / res as f32;
	let chunk_origin = chunk_coord.to_unwrapped_world_pos(chunk_size);

	// Vertical sampling range in world Y
	let y_min = -config.height_scale * 4.0;
	let y_max = config.height_scale * 4.0;
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
				grid[idx(x, y, z)] = sdf.as_ref().distance(Vec3::new(wx, wy, wz));
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
						let pos_local =
							interpolate_vertex(edge, cube_pos_local, cube_size, corners);
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
			calculate_sdf_normal(sdf.as_ref(), world).into()
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

/// Calculate normal from SDF gradient
fn calculate_sdf_normal(sdf: &dyn Sdf, p: Vec3) -> Vec3 {
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
	// feature_registry: Option<&FeatureRegistry>,
) -> Entity {
	// Use unwrapped coordinate for mesh generation to ensure seamless terrain
	let mesh = generate_chunk_mesh(&unwrapped_coord, chunk_size, resolution, config);
	let mesh_handle = meshes.add(mesh);

	// Make the origin chunk (0, 0) reddish for easy verification
	let is_origin_chunk = wrapped_coord.x == 0 && wrapped_coord.z == 0;
	let base_color = if is_origin_chunk {
		Color::hsla(46.0, 0.22, 0.62, 1.0) // brown
	} else {
		Color::hsla(46.0, 0.22, 0.62, 1.0) // brown
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
			Mesh3d(mesh_handle.clone()),
			MeshMaterial3d::<StandardMaterial>(material_handle.clone()),
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
