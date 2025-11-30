pub mod sparse_cubes;

use crate::cascade::CascadeChunk;
use crate::chunk::TerrainChunk;
// use crate::geography::FeatureRegistry;
use crate::sdf::Sign;
use crate::terrain::create_terrain_sdf;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use rayon::prelude::*;

/// CPU-based terrain mesh generator
pub struct CpuMeshGenerator;

impl CpuMeshGenerator {
	/// Generate a terrain mesh for a specific chunk by sampling an SDF
	/// Supports both heightfield (fast, no caves) and volumetric (marching cubes, supports caves)
	/// Returns None if the chunk is entirely above the terrain surface
	pub fn generate_chunk_mesh(
		cascade_chunk: &CascadeChunk,
		config: &TerrainConfig,
	) -> Option<Mesh> {
		// ---------- grid setup ---------------------------------------------------
		let chunk_size = cascade_chunk.size;
		let res = cascade_chunk.resolution();
		let cube_size = chunk_size / res as f32;
		let chunk_origin = cascade_chunk.origin;

		// ---------- grid setup ---------------------------------------------------
		// Grid resolution (sample points); cubes are (n-1) in each axis
		// Y is now treated the same as X and Z - a voxel cube
		let nx = res + 1;
		let ny = res + 1;
		let nz = res + 1;

		// Helper: linear index with X fastest, then Z, then Y (consistent)
		let idx = |x: usize, y: usize, z: usize| -> usize { (y * nz + z) * nx + x };

		// Scalar field samples
		let mut grid = vec![0.0f32; nx * ny * nz];

		// time the sampling
		let start_time = std::time::Instant::now();

		// ---------- sample SDF in world space (parallelized) --------------------
		// Parallelize over Z slices for sparse sampling using sign_uniform_on_y
		// Collect results per Z slice and merge sequentially
		let config_clone = config.clone();
		let z_slices: Vec<_> = (0..nz)
			.into_par_iter()
			.map(|z| {
				// Create SDF per thread to avoid Send/Sync issues
				let thread_sdf = create_terrain_sdf(&config_clone);
				let wz = chunk_origin.z + z as f32 * cube_size;
				let mut slice = vec![0.0f32; nx * ny];

				// For each x position, compute intervals and sample sparsely
				for x in 0..nx {
					let wx = chunk_origin.x + x as f32 * cube_size;
					// Get intervals for this (x, z) position
					let intervals = thread_sdf.as_ref().sign_uniform_on_y(wx, wz);

					// Iterate over intervals and sample/fill accordingly
					let mut y_current = 0;
					for interval in intervals.into_iter() {
						let start_time = std::time::Instant::now();
						let (y_min_world, y_max_world) = interval.open_range();
						let sign = interval.left.sign;

						// Convert world Y coordinates to grid indices
						// Clamp to chunk bounds
						let y_start_world = y_min_world.max(chunk_origin.y);
						let y_end_world = y_max_world.min(chunk_origin.y + chunk_size);

						let y_start =
							((y_start_world - chunk_origin.y) / cube_size).floor() as usize;
						let y_end = ((y_end_world - chunk_origin.y) / cube_size)
							.ceil()
							.min(ny as f32) as usize;

						// Only process if this interval overlaps with remaining Y values
						if y_start >= ny || y_current >= ny {
							break;
						}

						// Start from the current Y position or the interval start, whichever is later
						let y_begin = y_start.max(y_current);
						let y_finish = y_end.min(ny);

						if y_begin < y_finish {
							// Fill or sample based on sign
							match sign {
								Sign::Top | Sign::Bottom => {
									// Unknown/undefined sign - need to sample normally
									for yi in y_begin..y_finish {
										let wy = chunk_origin.y + yi as f32 * cube_size;
										let distance =
											thread_sdf.as_ref().distance(Vec3::new(wx, wy, wz));
										slice[yi * nx + x] = distance;
									}
								}
								Sign::Negative => {
									log::debug!("Negative sign for x: {:?}, z: {:?}", x, z);
									// Negative sign - fill with large negative value
									let fill_value = -1000.0;
									for yi in y_begin..y_finish {
										slice[yi * nx + x] = fill_value;
									}
								}
								Sign::Positive => {
									log::debug!("Positive sign for x: {:?}, z: {:?}", x, z);
									// Positive sign - fill with large positive value
									let fill_value = 1000.0;
									for yi in y_begin..y_finish {
										slice[yi * nx + x] = fill_value;
									}
								}
							}
						}

						let end_time = std::time::Instant::now();
						let duration = end_time.duration_since(start_time);
						log::debug!("Sparse sampling time for x: {:?}, z: {:?}: {:?}, covered y values: {:?}", x, z, duration, y_current);
						if y_current < ny {
							log::debug!("Covered y_start: {:?}, y_finish: {:?}", y_start, y_finish);
						}

						// Update current Y position to skip ahead
						y_current = y_finish;
						if y_current >= ny {
							break;
						}
					}

					// Fill any remaining Y values that weren't covered by intervals
					// This shouldn't happen with proper intervals, but handle it safely
					if y_current < ny {
						// Treat remaining as Top (unknown) and sample
						for yi in y_current..ny {
							let wy = chunk_origin.y + yi as f32 * cube_size;
							let distance = thread_sdf.as_ref().distance(Vec3::new(wx, wy, wz));
							slice[yi * nx + x] = distance;
						}
					}
				}

				(z, slice)
			})
			.collect();
		let end_time = std::time::Instant::now();
		let duration = end_time.duration_since(start_time);
		log::debug!("Sparse sampling time: {:?}", duration);

		// time the merging
		let start_time = std::time::Instant::now();
		// Merge slices into grid
		for (z, slice) in z_slices {
			for y in 0..ny {
				for x in 0..nx {
					grid[idx(x, y, z)] = slice[y * nx + x];
				}
			}
		}
		let end_time = std::time::Instant::now();
		let duration = end_time.duration_since(start_time);
		log::debug!("Merging time: {:?}", duration);

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
		let start_time = std::time::Instant::now();
		let cube_coords: Vec<_> = (0..cy)
			.flat_map(|y| (0..cz).flat_map(move |z| (0..cx).map(move |x| (x, y, z))))
			.collect();
		let end_time = std::time::Instant::now();
		let duration = end_time.duration_since(start_time);
		log::debug!("Cube coords time: {:?}", duration);

		// Capture grid as a slice for parallel access (read-only)
		let start_time = std::time::Instant::now();
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
		let end_time = std::time::Instant::now();
		let duration = end_time.duration_since(start_time);
		log::debug!("Cube results time: {:?}", duration);

		// Merge all cube results with proper index offsets
		let start_time = std::time::Instant::now();
		let mut vertices: Vec<[f32; 3]> = Vec::new();
		let mut indices: Vec<u32> = Vec::new();

		for (cube_vertices, cube_indices) in cube_results {
			let vertex_offset = vertices.len() as u32;
			vertices.extend(cube_vertices);
			indices.extend(cube_indices.iter().map(|&idx| idx + vertex_offset));
		}
		let end_time = std::time::Instant::now();
		let duration = end_time.duration_since(start_time);
		log::debug!("Merging cube results time: {:?}", duration);

		// time the normals
		let start_time = std::time::Instant::now();
		// ---------- Normals & UVs (parallelized) ---------------------------------
		// Normals: compute from voxel grid using finite differences
		// Vertices are in local space (relative to chunk_origin)
		let grid_slice: &[f32] = &grid;
		let normals: Vec<[f32; 3]> = vertices
			.par_iter()
			.map(|v| {
				// Convert vertex local position to grid coordinates
				let gx = (v[0] / cube_size).clamp(0.0, (nx - 1) as f32);
				let gy = (v[1] / cube_size).clamp(0.0, (ny - 1) as f32);
				let gz = (v[2] / cube_size).clamp(0.0, (nz - 1) as f32);

				// Get integer grid indices (truncate for now, could interpolate)
				let ix = gx as usize;
				let iy = gy as usize;
				let iz = gz as usize;

				// Compute finite differences using central differences where possible
				// ∂f/∂x = (f(x+1) - f(x-1)) / (2 * cube_size)
				let dx = if ix > 0 && ix < nx - 1 {
					let f_xp1 = grid_slice[idx(ix + 1, iy, iz)];
					let f_xm1 = grid_slice[idx(ix - 1, iy, iz)];
					(f_xp1 - f_xm1) / (2.0 * cube_size)
				} else if ix < nx - 1 {
					// Forward difference at left boundary
					let f_xp1 = grid_slice[idx(ix + 1, iy, iz)];
					let f_x = grid_slice[idx(ix, iy, iz)];
					(f_xp1 - f_x) / cube_size
				} else {
					// Backward difference at right boundary
					let f_x = grid_slice[idx(ix, iy, iz)];
					let f_xm1 = grid_slice[idx(ix - 1, iy, iz)];
					(f_x - f_xm1) / cube_size
				};

				// ∂f/∂y = (f(y+1) - f(y-1)) / (2 * cube_size)
				let dy = if iy > 0 && iy < ny - 1 {
					let f_yp1 = grid_slice[idx(ix, iy + 1, iz)];
					let f_ym1 = grid_slice[idx(ix, iy - 1, iz)];
					(f_yp1 - f_ym1) / (2.0 * cube_size)
				} else if iy < ny - 1 {
					// Forward difference at bottom boundary
					let f_yp1 = grid_slice[idx(ix, iy + 1, iz)];
					let f_y = grid_slice[idx(ix, iy, iz)];
					(f_yp1 - f_y) / cube_size
				} else {
					// Backward difference at top boundary
					let f_y = grid_slice[idx(ix, iy, iz)];
					let f_ym1 = grid_slice[idx(ix, iy - 1, iz)];
					(f_y - f_ym1) / cube_size
				};

				// ∂f/∂z = (f(z+1) - f(z-1)) / (2 * cube_size)
				let dz = if iz > 0 && iz < nz - 1 {
					let f_zp1 = grid_slice[idx(ix, iy, iz + 1)];
					let f_zm1 = grid_slice[idx(ix, iy, iz - 1)];
					(f_zp1 - f_zm1) / (2.0 * cube_size)
				} else if iz < nz - 1 {
					// Forward difference at front boundary
					let f_zp1 = grid_slice[idx(ix, iy, iz + 1)];
					let f_z = grid_slice[idx(ix, iy, iz)];
					(f_zp1 - f_z) / cube_size
				} else {
					// Backward difference at back boundary
					let f_z = grid_slice[idx(ix, iy, iz)];
					let f_zm1 = grid_slice[idx(ix, iy, iz - 1)];
					(f_z - f_zm1) / cube_size
				};

				// Normalize the gradient to get the normal
				let grad = Vec3::new(dx, dy, dz);
				let len = grad.length();
				if len > 0.0001 {
					(grad / len).into()
				} else {
					Vec3::Y.into() // Fallback to up if gradient is too small
				}
			})
			.collect();
		let end_time = std::time::Instant::now();
		let duration = end_time.duration_since(start_time);
		log::debug!("Normals time: {:?}", duration);

		// Simple tiled UVs (local X/Z across the chunk)
		let start_time = std::time::Instant::now();
		let uvs: Vec<[f32; 2]> =
			vertices.par_iter().map(|v| [v[0] / chunk_size, v[2] / chunk_size]).collect();
		let end_time = std::time::Instant::now();
		let duration = end_time.duration_since(start_time);
		log::debug!("UVs time: {:?}", duration);

		// ---------- Mesh ---------------------------------------------------------
		let mut mesh = Mesh::new(
			bevy::mesh::PrimitiveTopology::TriangleList,
			bevy::asset::RenderAssetUsages::RENDER_WORLD,
		);
		mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
		mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
		mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
		mesh.insert_indices(bevy::mesh::Indices::U32(indices));
		Some(mesh)
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
		let start_time = std::time::Instant::now();
		let Some(mesh) = Self::generate_chunk_mesh(&cascade_chunk, config) else {
			// Chunk is entirely above terrain, don't spawn it
			log::debug!(
				"Skipping chunk at origin {:?} - entirely above terrain",
				cascade_chunk.origin
			);
			// Return a dummy entity that will be cleaned up
			return commands.spawn_empty().id();
		};
		let mesh_handle = meshes.add(mesh);
		let end_time = std::time::Instant::now();
		let duration = end_time.duration_since(start_time);
		log::info!("Mesh time: {:?}", duration);

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
			cascade_chunk.resolution()
		);

		entity
	}
}
