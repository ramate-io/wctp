// =================================================================================================
// STAGE 4: PREFIX ADD
// =================================================================================================
// Add block prefixes to local offsets

use bevy::render::{
	render_resource::{BindGroupLayout, Buffer, CommandEncoder, ComputePassDescriptor, ComputePipeline},
	renderer::RenderDevice,
};

use crate::pipeline::proc::bind_groups::create_bind_group;

/// Stage 4: Add block prefixes to local offsets
pub fn stage_prefix_add(
	device: &RenderDevice,
	layout: &BindGroupLayout,
	pipeline: &ComputePipeline,
	block_count: u32,
	tri_offset: &Buffer,
	block_prefix: &Buffer,
	encoder: &mut CommandEncoder,
) {
	let bind = create_bind_group(
		device,
		"mc_prefix_add_bind",
		layout,
		&[tri_offset, block_prefix],
	);

	let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
	pass.set_pipeline(pipeline);
	pass.set_bind_group(0, &bind, &[]);
	pass.dispatch_workgroups(block_count, 1, 1);
}

