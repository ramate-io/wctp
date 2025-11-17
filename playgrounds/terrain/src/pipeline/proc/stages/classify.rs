// =================================================================================================
// STAGE 1: CLASSIFY
// =================================================================================================
// Classify voxels and compute cube indices
//
// WGSL binding contract:
//   @group(0) @binding(0) var<uniform> sampling : Sampling3D;
//   @group(0) @binding(1) var<uniform> terrain_config : TerrainConfig;
//   @group(0) @binding(2) var<uniform> bounds : Bounds;
//   @group(0) @binding(3) var<uniform> seed : i32;
//   @group(0) @binding(4) var<storage, read_write> cube_index : array<u32>;
//   @group(0) @binding(5) var<storage, read_write> tri_counts : array<u32>;

use bevy::{
	prelude::UVec3,
	render::{
		render_resource::{
			BindGroupLayout, Buffer, CachedComputePipelineId, CommandEncoder,
			ComputePassDescriptor, PipelineCache,
		},
		renderer::RenderDevice,
	},
};

use crate::pipeline::proc::bind_groups::{create_bind_group, create_storage_layout_entry, create_uniform_layout_entry};

/// Stage for classifying voxels and computing cube indices.
/// Uses pipeline ID from resource instead of creating pipelines on the fly.
pub struct ClassifyStage;

impl ClassifyStage {
	/// Create the bind group layout for the classify stage.
	pub fn create_layout(device: &RenderDevice) -> BindGroupLayout {
		device.create_bind_group_layout(
			Some("mc_classify_layout"),
			&[
				create_uniform_layout_entry(0),        // sampling
				create_uniform_layout_entry(1),        // terrain_config
				create_uniform_layout_entry(2),        // bounds
				create_uniform_layout_entry(3),        // seed
				create_storage_layout_entry(4, false), // cube_index
				create_storage_layout_entry(5, false), // tri_counts
			],
		)
	}

	/// Execute the classify stage using pipeline ID from resource.
	pub fn execute(
		device: &RenderDevice,
		pipeline_cache: &PipelineCache,
		layout: &BindGroupLayout,
		pipeline_id: CachedComputePipelineId,
		dispatch: UVec3,
		sampling_buf: &Buffer,
		cfg_buf: &Buffer,
		bounds_buf: &Buffer,
		seed_buf: &Buffer,
		cube_index: &Buffer,
		tri_counts: &Buffer,
		encoder: &mut CommandEncoder,
	) {
		// Get the actual pipeline from the cache
		let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) else {
			log::warn!("Classify pipeline not ready yet");
			return;
		};

		let bind = create_bind_group(
			device,
			"mc_classify_bind",
			layout,
			&[sampling_buf, cfg_buf, bounds_buf, seed_buf, cube_index, tri_counts],
		);

		let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
		pass.set_pipeline(pipeline);
		pass.set_bind_group(0, &bind, &[]);
		pass.dispatch_workgroups(dispatch.x, dispatch.y, dispatch.z);
	}
}
