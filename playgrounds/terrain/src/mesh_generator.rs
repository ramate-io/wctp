use crate::chunk::ChunkCoord;
use crate::cpu::CpuMeshGenerator;
use crate::gpu::GpuMeshGenerator;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
use bevy::render::render_resource::PipelineCache;
use bevy::render::renderer::{RenderDevice, RenderQueue};

/// Mesh generation mode - selects between CPU and GPU implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum MeshGenerationMode {
	/// Use CPU-based marching cubes
	Cpu,
	/// Use GPU-based marching cubes
	Gpu,
}

impl Default for MeshGenerationMode {
	fn default() -> Self {
		MeshGenerationMode::Cpu
	}
}

/// Unified interface for spawning terrain chunks
pub struct MeshGenerator;

impl MeshGenerator {
	/// Spawn a terrain chunk entity using the selected generation mode
	pub fn spawn_chunk(
		mode: MeshGenerationMode,
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
		wrapped_coord: ChunkCoord,
		unwrapped_coord: ChunkCoord,
		chunk_size: f32,
		world_size_chunks: i32,
		resolution: usize,
		config: &TerrainConfig,
		// GPU resources (only used if mode is Gpu)
		device: Option<&RenderDevice>,
		queue: Option<&RenderQueue>,
		pipeline_cache: Option<&mut PipelineCache>,
		asset_server: Option<&AssetServer>,
		shaders: Option<&Assets<bevy::render::render_resource::Shader>>,
	) -> Entity {
		match mode {
			MeshGenerationMode::Cpu => CpuMeshGenerator::spawn_chunk(
				commands,
				meshes,
				materials,
				wrapped_coord,
				unwrapped_coord,
				chunk_size,
				world_size_chunks,
				resolution,
				config,
			),
			MeshGenerationMode::Gpu => {
				let device = device.expect("RenderDevice required for GPU mode");
				let queue = queue.expect("RenderQueue required for GPU mode");
				let pipeline_cache = pipeline_cache.expect("PipelineCache required for GPU mode");
				let asset_server = asset_server.expect("AssetServer required for GPU mode");
				let shaders = shaders.expect("Shaders required for GPU mode");

				GpuMeshGenerator::spawn_chunk(
					commands,
					meshes,
					materials,
					wrapped_coord,
					unwrapped_coord,
					chunk_size,
					world_size_chunks,
					resolution,
					config,
					device,
					queue,
					pipeline_cache,
					asset_server,
					shaders,
				)
			}
		}
	}
}
