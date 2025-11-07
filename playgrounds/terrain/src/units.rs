use crate::chunk::{ChunkConfig, ChunkCoord, TerrainChunk};
use crate::geography::FeatureRegistry;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

/// Get the terrain height at a world position
/// Uses the same noise generation as terrain mesh creation for consistency
pub fn get_terrain_height_at(
	world_pos: Vec3,
	chunk_query: &Query<(&TerrainChunk, &Transform)>,
	chunk_config: &ChunkConfig,
	terrain_config: &TerrainConfig,
	perlin: &Perlin,
	feature_registry: Option<&FeatureRegistry>,
) -> Option<f32> {
	// Find which chunk this position is in
	let (chunk_coord, _) = ChunkCoord::from_world_pos(
		world_pos,
		chunk_config.chunk_size,
		chunk_config.world_size_chunks,
	);

	// Find the terrain chunk entity
	log::debug!("Looking for chunk at coord ({}, {})", chunk_coord.x, chunk_coord.z);
	for (chunk, _transform) in chunk_query.iter() {
		if chunk.coord == chunk_coord {
			log::debug!("Found matching chunk!");
			// Note: world_pos.y is ignored, we're only using x and z for terrain lookup
			// Use world_pos directly since we only need x and z
			let world_x = world_pos.x;
			let world_z = world_pos.z;

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

			height = (height / max_value) * terrain_config.height_scale;

			// Apply geographic features
			if let Some(registry) = feature_registry {
				height = registry.apply_features(world_x, world_z, height, terrain_config);
			}

			log::debug!("Terrain height at ({}, {}) = {}", world_x, world_z, height);
			return Some(height);
		}
	}

	log::warn!("No matching chunk found for coord ({}, {})", chunk_coord.x, chunk_coord.z);
	None
}

/// Attach a mesh to terrain by positioning it so it sits on the surface
/// For a cuboid, we know its half-extents, so we can sample terrain in that area
pub fn attach_cuboid_to_terrain(
	half_size: Vec3,
	world_x: f32,
	world_z: f32,
	chunk_query: &Query<(&TerrainChunk, &Transform)>,
	chunk_config: &ChunkConfig,
	terrain_config: &TerrainConfig,
	perlin: &Perlin,
	feature_registry: Option<&FeatureRegistry>,
) -> Option<Vec3> {
	// Sample terrain heights in the area where the mesh will be placed
	// Sample at multiple points to find the lowest terrain point in this area
	let sample_count = 5; // Sample a 5x5 grid
	let mut min_terrain_height = f32::MAX;
	let mut sample_count_found = 0;

	log::info!("Attaching cuboid with half_size {:?} at ({}, {})", half_size, world_x, world_z);

	for i in 0..sample_count {
		for j in 0..sample_count {
			// Calculate sample position within mesh footprint
			let t_x = i as f32 / (sample_count - 1) as f32; // 0.0 to 1.0
			let t_z = j as f32 / (sample_count - 1) as f32; // 0.0 to 1.0

			// Sample from -half_size to +half_size in x and z
			let sample_x = world_x - half_size.x + t_x * (half_size.x * 2.0);
			let sample_z = world_z - half_size.z + t_z * (half_size.z * 2.0);

			if let Some(height) = get_terrain_height_at(
				Vec3::new(sample_x, 0.0, sample_z),
				chunk_query,
				chunk_config,
				terrain_config,
				perlin,
				feature_registry,
			) {
				min_terrain_height = min_terrain_height.min(height);
				sample_count_found += 1;
			}
		}
	}

	log::info!(
		"Sampled {} points, found {} valid heights, min_terrain_height = {}",
		sample_count * sample_count,
		sample_count_found,
		min_terrain_height
	);

	if min_terrain_height == f32::MAX {
		log::error!("No valid terrain found for attachment!");
		return None; // No valid terrain found
	}

	// Position the mesh so its bottom face is below the lowest terrain point in its footprint
	// The mesh's bottom face is at -half_size.y relative to its origin
	// We want: y_position - half_size.y < min_terrain_height
	// So: y_position < min_terrain_height + half_size.y
	// Position it so bottom is just below terrain (with small offset)
	let offset = 0.1; // Small offset to ensure bottom is below terrain
	let y_position = min_terrain_height + half_size.y - offset;
	let final_position = Vec3::new(world_x, y_position, world_z);

	let cube_bottom = y_position - half_size.y;
	let cube_top = y_position + half_size.y;
	log::info!(
		"Positioned cuboid at {:?} (y_position = {}, min_terrain = {}, cube_bottom = {}, cube_top = {})",
		final_position,
		y_position,
		min_terrain_height,
		cube_bottom,
		cube_top
	);

	Some(final_position)
}

/// Marker component to ensure we only spawn the cube once
#[derive(Component)]
pub struct AttachedCubeSpawned;

/// Spawn a cubic mesh attached to the terrain
pub fn spawn_attached_cube(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	chunk_query: Query<(&TerrainChunk, &Transform)>,
	chunk_config: Res<ChunkConfig>,
	terrain_config: Res<TerrainConfig>,
	feature_registry: Option<Res<FeatureRegistry>>,
	spawned_query: Query<&AttachedCubeSpawned>,
) {
	// Only spawn once
	if !spawned_query.is_empty() {
		return;
	}

	// Only spawn if we have chunks loaded
	if chunk_query.is_empty() {
		log::debug!("No chunks loaded yet, waiting...");
		return;
	}
	// Create a big, obvious cube mesh
	let cube_size = 20.0; // Much bigger
	let half_size = cube_size * 0.5;
	let cube_mesh_handle = meshes.add(Cuboid::new(cube_size, cube_size, cube_size));

	// Create blue material with better reflectance
	let material = materials.add(StandardMaterial {
		base_color: Color::srgb(0.3, 0.3, 0.8), // Blue
		metallic: 0.0,
		perceptual_roughness: 0.4, // Less rough for more light reflection/bounce
		..default()
	});

	// Position near origin
	let world_x = 0.0;
	let world_z = 0.0;

	// Create Perlin noise generator
	let perlin = Perlin::new(terrain_config.seed);

	log::info!(
		"Spawning attached cube: size={}, half_size={}, world_pos=({}, {})",
		cube_size,
		half_size,
		world_x,
		world_z
	);
	log::info!("Chunk query has {} chunks", chunk_query.iter().count());

	// Attach the cube to terrain using its half-extents
	if let Some(position) = attach_cuboid_to_terrain(
		Vec3::new(half_size, half_size, half_size),
		world_x,
		world_z,
		&chunk_query,
		&chunk_config,
		&terrain_config,
		&perlin,
		feature_registry.as_deref(),
	) {
		log::info!("Successfully attached cube at position {:?}", position);
		commands.spawn((
			Mesh3d(cube_mesh_handle),
			MeshMaterial3d(material),
			Transform::from_translation(position),
			AttachedCubeSpawned,
		));
	} else {
		log::error!("Failed to attach cube to terrain!");
	}
}
