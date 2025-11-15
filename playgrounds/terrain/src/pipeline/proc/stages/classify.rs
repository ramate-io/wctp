// =================================================================================================
// STAGE 1: CLASSIFY
// =================================================================================================
// Classify voxels and compute cube indices

use bevy::{
	prelude::UVec3,
	render::{
		render_resource::{BindGroupLayout, Buffer, CommandEncoder, ComputePassDescriptor, ComputePipeline},
		renderer::RenderDevice,
	},
};

use crate::pipeline::proc::bind_groups::create_bind_group;

/// Stage 1: Classify voxels and compute cube indices
pub fn stage_classify(
	device: &RenderDevice,
	layout: &BindGroupLayout,
	pipeline: &ComputePipeline,
	dispatch: UVec3,
	sampling_buf: &Buffer,
	cfg_buf: &Buffer,
	bounds_buf: &Buffer,
	seed_buf: &Buffer,
	cube_index: &Buffer,
	tri_counts: &Buffer,
	encoder: &mut CommandEncoder,
) {
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

