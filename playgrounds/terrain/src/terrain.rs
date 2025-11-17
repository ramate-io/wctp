use crate::chunk::{ChunkCoord, TerrainChunk};
use crate::pipeline::proc::{Bounds, GpuMarchingCubesPipeline, Sampling3D, TerrainMeshSpawner};
// use crate::geography::FeatureRegistry;
use crate::sdf::{
	region::{
		affine::RegionAffineModulation, branching::BranchingPlan, grading::RegionGradingModulation,
		rounding::RegionRoundingModulation, CircleRegion, RectRegion, Region2D, RegionNoise,
	},
	Difference, Ellipse3d, PerlinTerrainSdf, Sdf, TubeSdf,
};
use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
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
	let branch_plan = BranchingPlan::new(big_valley_sdf, Perlin::new(config.seed), 5, 2);

	let modulations = branch_plan.generate_regions();

	for modulation in modulations {
		sdf.add_elevation_modulation(Box::new(modulation));
	}

	let road_sdf = RegionRoundingModulation::new(
		Region2D::Rect(RectRegion {
			center: Vec2::new(0.0, 0.0),
			half_extents: Vec2::new(80.0, 1.0),
			round: 0.1,
		}),
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
		Region2D::Rect(RectRegion {
			center: Vec2::new(20.0, 20.0),
			half_extents: Vec2::new(20.0, 1.0),
			round: 0.01,
		}),
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

/// Generate a terrain mesh for a specific chunk using GPU-based Marching Cubes
pub fn generate_chunk_mesh(
	chunk_coord: &ChunkCoord,
	chunk_size: f32,
	resolution: usize,
	config: &TerrainConfig,
	device: &RenderDevice,
	queue: &RenderQueue,
	pipeline_cache: &mut bevy::render::render_resource::PipelineCache,
	asset_server: &AssetServer,
	shaders: &Assets<bevy::render::render_resource::Shader>,
) -> Mesh {
	generate_chunk_mesh_gpu(
		chunk_coord,
		chunk_size,
		resolution,
		config,
		device,
		queue,
		pipeline_cache,
		asset_server,
		shaders,
	)
}

/// Generate a terrain mesh using GPU-based Marching Cubes
pub fn generate_chunk_mesh_gpu(
	chunk_coord: &ChunkCoord,
	chunk_size: f32,
	res: usize,
	config: &TerrainConfig,
	device: &RenderDevice,
	queue: &RenderQueue,
	pipeline_cache: &mut bevy::render::render_resource::PipelineCache,
	asset_server: &AssetServer,
	shaders: &Assets<bevy::render::render_resource::Shader>,
) -> Mesh {
	let chunk_origin = chunk_coord.to_unwrapped_world_pos(chunk_size);

	// Vertical sampling range in world Y
	let y_min = -config.height_scale * 2.0;
	let y_max = config.height_scale * 2.0;
	let y_range = y_max - y_min;

	// Calculate vertical resolution to match horizontal resolution
	let cube_size = chunk_size / res as f32;
	let y_cells = (y_range / cube_size).ceil().max(1.0) as usize;

	// Set up GPU pipeline parameters
	let sampling = Sampling3D {
		chunk_origin,
		chunk_size: Vec3::new(chunk_size, y_range, chunk_size),
		resolution: UVec3::new(res as u32, y_cells as u32, res as u32),
	};

	let bounds = Bounds {
		enabled: 0, // No bounds restriction for now
		min: Vec2::ZERO,
		max: Vec2::ZERO,
	};

	// Create GPU pipeline
	let pipeline = GpuMarchingCubesPipeline::new(
		device,
		pipeline_cache,
		asset_server,
		shaders,
		sampling,
		config,
		bounds,
		config.seed as i32,
	);

	// Compute mesh on GPU
	let gpu_data = pipeline.compute(device, queue);

	// Convert to Bevy Mesh
	TerrainMeshSpawner::mesh_from_gpu_data(&gpu_data)
}

/// Calculate normal from SDF gradient
pub(crate) fn calculate_sdf_normal(sdf: &dyn Sdf, p: Vec3) -> Vec3 {
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
	device: &RenderDevice,
	queue: &RenderQueue,
	pipeline_cache: &mut bevy::render::render_resource::PipelineCache,
	asset_server: &AssetServer,
	shaders: &Assets<bevy::render::render_resource::Shader>,
	// feature_registry: Option<&FeatureRegistry>,
) -> Entity {
	// Use unwrapped coordinate for mesh generation to ensure seamless terrain
	let mesh = generate_chunk_mesh(
		&unwrapped_coord,
		chunk_size,
		resolution,
		config,
		device,
		queue,
		pipeline_cache,
		asset_server,
		shaders,
	);
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
