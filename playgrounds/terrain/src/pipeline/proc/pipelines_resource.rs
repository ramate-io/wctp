// =================================================================================================
// MARCHING CUBES PIPELINES RESOURCE
// =================================================================================================
// Stores all compute pipeline IDs for the marching cubes GPU pipeline.
// Pipelines are created once in RenderApp and stored here for use in compute dispatch.

use bevy::{
	prelude::Resource,
	render::render_resource::CachedComputePipelineId,
};

/// Resource containing all compute pipeline IDs for the marching cubes GPU pipeline.
/// Created once in RenderApp startup, then used for all compute dispatches.
#[derive(Resource, Clone)]
pub struct MarchingCubesPipelines {
	pub classify: CachedComputePipelineId,
	pub prefix_local: CachedComputePipelineId,
	pub prefix_block: CachedComputePipelineId,
	pub prefix_add: CachedComputePipelineId,
	pub mesh: CachedComputePipelineId,
}

