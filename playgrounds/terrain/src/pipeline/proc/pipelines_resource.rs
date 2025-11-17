// =================================================================================================
// MARCHING CUBES PIPELINES RESOURCE
// =================================================================================================
// Stores all compute pipelines for the marching cubes GPU pipeline.
// Pipelines are created once in RenderApp and stored here for use in compute dispatch.

use bevy::{prelude::*, render::render_resource::ComputePipeline};

/// Resource containing all compute pipelines for the marching cubes GPU pipeline.
/// Created once in RenderApp after pipelines are compiled, then used for all compute dispatches.
#[derive(Resource, Debug)]
pub struct MarchingCubesPipelines {
	pub classify: ComputePipeline,
	pub prefix_local: ComputePipeline,
	pub prefix_block: ComputePipeline,
	pub prefix_add: ComputePipeline,
	pub mesh: ComputePipeline,
}

/// Resource containing pipeline IDs used for polling until pipelines are ready.
/// This is temporary and removed once pipelines are loaded.
#[derive(Resource, Debug)]
pub struct MarchingCubesPipelineIds {
	pub classify: bevy::render::render_resource::CachedComputePipelineId,
	pub prefix_local: bevy::render::render_resource::CachedComputePipelineId,
	pub prefix_block: bevy::render::render_resource::CachedComputePipelineId,
	pub prefix_add: bevy::render::render_resource::CachedComputePipelineId,
	pub mesh: bevy::render::render_resource::CachedComputePipelineId,
}

/// Resource containing shader handles for checking if shaders are loaded.
/// This is temporary and removed once pipelines are loaded.
#[derive(Resource, Debug)]
pub struct MarchingCubesShaderHandles {
	pub classify: Handle<Shader>,
	pub prefix_local: Handle<Shader>,
	pub prefix_block: Handle<Shader>,
	pub prefix_add: Handle<Shader>,
	pub mesh: Handle<Shader>,
}
