use crate::chunk::{ChunkCoord, TerrainChunk};
use crate::pipeline::proc::{Bounds, GpuMarchingCubesPipeline, Sampling3D, TerrainMeshSpawner};
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::render_resource::PipelineCache;

/// GPU-based terrain mesh generator
pub struct GpuMeshGenerator;

impl GpuMeshGenerator {
	/// Generate a terrain mesh for a specific chunk using GPU-based Marching Cubes
	pub fn generate_chunk_mesh(
		chunk_coord: &ChunkCoord,
		chunk_size: f32,
		resolution: usize,
		config: &TerrainConfig,
		device: &RenderDevice,
		queue: &RenderQueue,
		pipeline_cache: &mut PipelineCache,
		asset_server: &AssetServer,
		shaders: &Assets<bevy::render::render_resource::Shader>,
	) -> Mesh {
		let chunk_origin = chunk_coord.to_unwrapped_world_pos(chunk_size);

		// Vertical sampling range in world Y
		let y_min = -config.height_scale * 2.0;
		let y_max = config.height_scale * 2.0;
		let y_range = y_max - y_min;

		// Calculate vertical resolution to match horizontal resolution
		let cube_size = chunk_size / resolution as f32;
		let y_cells = (y_range / cube_size).ceil().max(1.0) as usize;

		// Set up GPU pipeline parameters
		let sampling = Sampling3D {
			chunk_origin,
			chunk_size: Vec3::new(chunk_size, y_range, chunk_size),
			resolution: UVec3::new(resolution as u32, y_cells as u32, resolution as u32),
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

	/// Spawn a terrain chunk entity using GPU mesh generation
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
		pipeline_cache: &mut PipelineCache,
		asset_server: &AssetServer,
		shaders: &Assets<bevy::render::render_resource::Shader>,
	) -> Entity {
		// Use unwrapped coordinate for mesh generation to ensure seamless terrain
		let mesh = Self::generate_chunk_mesh(
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
			"Spawned chunk (GPU) wrapped=({}, {}) unwrapped=({}, {}) at world position {:?} with resolution {}",
			wrapped_coord.x,
			wrapped_coord.z,
			unwrapped_coord.x,
			unwrapped_coord.z,
			world_pos,
			resolution
		);

		entity
	}
}

