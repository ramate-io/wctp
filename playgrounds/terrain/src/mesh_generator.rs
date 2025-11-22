use crate::cascade::CascadeChunk;
use crate::cpu::CpuMeshGenerator;
use crate::gpu::GpuMeshGenerator;
use crate::pipeline::proc::pipelines_resource::MarchingCubesPipelines;
use crate::terrain::TerrainConfig;
use bevy::prelude::*;
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
		cascade_chunk: CascadeChunk,
		config: &TerrainConfig,
		// GPU resources (only used if mode is Gpu)
		device: Option<&RenderDevice>,
		queue: Option<&RenderQueue>,
		pipelines: Option<&MarchingCubesPipelines>,
	) -> Entity {
		match mode {
			MeshGenerationMode::Cpu => {
				CpuMeshGenerator::spawn_chunk(commands, meshes, materials, cascade_chunk, config)
			}
			MeshGenerationMode::Gpu => {
				let device = device.expect("RenderDevice required for GPU mode");
				let queue = queue.expect("RenderQueue required for GPU mode");
				let pipelines = pipelines.expect("MarchingCubesPipelines required for GPU mode");

				GpuMeshGenerator::spawn_chunk(
					commands,
					meshes,
					materials,
					cascade_chunk,
					config,
					device,
					queue,
					pipelines,
				)
			}
		}
	}
}
