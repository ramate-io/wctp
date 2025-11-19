use crate::cascade::CascadeChunk;
use crate::chunk::TerrainChunk;
// use crate::geography::FeatureRegistry;
use crate::sdf::Sdf;
use crate::terrain::create_terrain_sdf;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use rayon::prelude::*;

/// CPU-based terrain mesh generator
pub struct CpuMeshGenerator;

impl CpuMeshGenerator {
	/// Generate a terrain mesh for a specific chunk by sampling an SDF
	/// Supports both heightfield (fast, no caves) and volumetric (marching cubes, supports caves)
	pub fn generate_chunk_mesh(
		cascade_chunk: &CascadeChunk,
		config: &TerrainConfig,
		// feature_registry: Option<&FeatureRegistry>,
	) -> Mesh {
		Self::generate_chunk_mesh_volumetric(cascade_chunk, config)
	}

	fn generate_chunk_mesh_volumetric(
		cascade_chunk: &CascadeChunk,
		config: &TerrainConfig,
		// feature_registry: Option<&FeatureRegistry>,
	) -> Mesh {
		// Use the same SDF creation logic
		let sdf = create_terrain_sdf(config);

		// ---------- grid setup ---------------------------------------------------
		let chunk_size = cascade_chunk.size;
		let res = cascade_chunk.resolution;
		let cube_size = chunk_size / res as f32;
		let chunk_origin = cascade_chunk.origin;

		// Grid resolution (sample points); cubes are (n-1) in each axis
		// Y is now treated the same as X and Z - a voxel cube
		let nx = res + 1;
		let ny = res + 1;
		let nz = res + 1;

		// Helper: linear index with X fastest, then Z, then Y (consistent)
		let idx = |x: usize, y: usize, z: usize| -> usize { (y * nz + z) * nx + x };

		// Scalar field samples
		let mut grid = vec![0.0f32; nx * ny * nz];

		// ---------- sample SDF in world space (parallelized) --------------------
		// Parallelize over Y slices for better cache locality
		// Collect results per Y slice and merge sequentially
		let config_clone = config.clone();
		let y_slices: Vec<_> = (0..ny)
			.into_par_iter()
			.map(|y| {
				// Create SDF per thread to avoid Send/Sync issues
				let thread_sdf = create_terrain_sdf(&config_clone);
				// Y is now treated the same as X and Z - relative to chunk origin
				let wy = chunk_origin.y + y as f32 * cube_size;
				let mut slice = vec![0.0f32; nx * nz];
				for z in 0..nz {
					let wz = chunk_origin.z + z as f32 * cube_size;
					for x in 0..nx {
						let wx = chunk_origin.x + x as f32 * cube_size;
						slice[z * nx + x] = thread_sdf.as_ref().distance(Vec3::new(wx, wy, wz));
					}
				}
				(y, slice)
			})
			.collect();

		// Merge slices into grid
		for (y, slice) in y_slices {
			for z in 0..nz {
				for x in 0..nx {
					grid[idx(x, y, z)] = slice[z * nx + x];
				}
			}
		}

		// ---------- Marching Cubes (parallelized) --------------------------------
		use crate::marching_cubes::{get_cube_index, interpolate_vertex, TRIANGULATIONS};

		// Number of cubes along each axis
		let cx = nx - 1;
		let cy = ny - 1;
		let cz = nz - 1;

		// Process cubes in parallel, collecting vertices and indices per cube
		// We'll merge them with proper index offsets afterward
		// SAFETY: We're only reading from grid, and each thread reads different indices
		// Flatten cube coordinates into a single iterator
		let cube_coords: Vec<_> = (0..cy)
			.flat_map(|y| (0..cz).flat_map(move |z| (0..cx).map(move |x| (x, y, z))))
			.collect();

		// Capture grid as a slice for parallel access (read-only)
		let grid_slice: &[f32] = &grid;
		let cube_results: Vec<_> = cube_coords
			.into_par_iter()
			.filter_map(|(x, y, z)| {
				// Corner scalar values (standard MC corner ordering assumed by your helpers)
				// Inline index calculation: (y * nz + z) * nx + x
				let corners = [
					grid_slice[(y * nz + z) * nx + x],                   // 0 (0,0,0)
					grid_slice[(y * nz + z) * nx + (x + 1)],             // 1 (1,0,0)
					grid_slice[(y * nz + (z + 1)) * nx + (x + 1)],       // 2 (1,0,1)
					grid_slice[(y * nz + (z + 1)) * nx + x],             // 3 (0,0,1)
					grid_slice[((y + 1) * nz + z) * nx + x],             // 4 (0,1,0)
					grid_slice[((y + 1) * nz + z) * nx + (x + 1)],       // 5 (1,1,0)
					grid_slice[((y + 1) * nz + (z + 1)) * nx + (x + 1)], // 6 (1,1,1)
					grid_slice[((y + 1) * nz + (z + 1)) * nx + x],       // 7 (0,1,1)
				];

				let cube_index = get_cube_index(corners);
				if cube_index == 0 || cube_index == 255 {
					return None; // fully inside or outside
				}

				// Local-space cube origin (all dimensions relative to chunk origin)
				let cube_pos_local =
					Vec3::new(x as f32 * cube_size, y as f32 * cube_size, z as f32 * cube_size);

				// Per-cube edge vertex cache (12 edges)
				let mut edge_vert: [Option<u32>; 12] = [None; 12];

				let mut cube_vertices = Vec::new();
				let mut cube_indices = Vec::new();

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
						let v_index = cube_vertices.len() as u32;
						cube_vertices.push([pos_local.x, pos_local.y, pos_local.z]);
						edge_vert[edge] = Some(v_index);
						v_index
					};

					let v0 = get_vert(e0 as usize);
					let v1 = get_vert(e1 as usize);
					let v2 = get_vert(e2 as usize);

					cube_indices.extend_from_slice(&[v0, v1, v2]);
					i += 3;
				}

				if cube_vertices.is_empty() {
					None
				} else {
					Some((cube_vertices, cube_indices))
				}
			})
			.collect();

		// Merge all cube results with proper index offsets
		let mut vertices: Vec<[f32; 3]> = Vec::new();
		let mut indices: Vec<u32> = Vec::new();

		for (cube_vertices, cube_indices) in cube_results {
			let vertex_offset = vertices.len() as u32;
			vertices.extend(cube_vertices);
			indices.extend(cube_indices.iter().map(|&idx| idx + vertex_offset));
		}

		// ---------- Normals & UVs (parallelized) ---------------------------------
		// Normals: sample SDF gradient in WORLD space for shading correctness.
		// Convert local coordinates back to world by adding chunk_origin.
		let normals: Vec<[f32; 3]> = vertices
			.par_iter()
			.map(|v| {
				let world =
					Vec3::new(v[0] + chunk_origin.x, v[1] + chunk_origin.y, v[2] + chunk_origin.z);
				Self::calculate_sdf_normal(sdf.as_ref(), world).into()
			})
			.collect();

		// Simple tiled UVs (local X/Z across the chunk)
		let uvs: Vec<[f32; 2]> =
			vertices.par_iter().map(|v| [v[0] / chunk_size, v[2] / chunk_size]).collect();

		// ---------- Mesh ---------------------------------------------------------
		let mut mesh = Mesh::new(
			bevy::mesh::PrimitiveTopology::TriangleList,
			bevy::asset::RenderAssetUsages::RENDER_WORLD,
		);
		mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
		mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
		mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
		mesh.insert_indices(bevy::mesh::Indices::U32(indices));
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

	/// Spawn a terrain chunk entity using CPU mesh generation
	pub fn spawn_chunk(
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
		cascade_chunk: CascadeChunk,
		config: &TerrainConfig,
		// feature_registry: Option<&FeatureRegistry>,
	) -> Entity {
		// Generate mesh using cascade chunk
		let mesh = Self::generate_chunk_mesh(&cascade_chunk, config);
		let mesh_handle = meshes.add(mesh);

		// Make the origin chunk (0, 0, 0) brown for easy verification
		let is_origin_chunk = cascade_chunk.origin == Vec3::ZERO;
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

		// Use cascade chunk origin for world position
		// Note: mesh vertices are in local space relative to chunk origin
		let world_pos = cascade_chunk.origin;

		let entity = commands
			.spawn((
				TerrainChunk { chunk: cascade_chunk },
				Mesh3d(mesh_handle.clone()),
				MeshMaterial3d::<StandardMaterial>(material_handle.clone()),
				Transform::from_translation(world_pos),
			))
			.id();

		log::debug!(
			"Spawned chunk (CPU) at origin {:?} with size {} and resolution {}",
			cascade_chunk.origin,
			cascade_chunk.size,
			cascade_chunk.resolution
		);

		entity
	}
}
