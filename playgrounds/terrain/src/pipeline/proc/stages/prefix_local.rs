// =================================================================================================
// STAGE 2: PREFIX LOCAL
// =================================================================================================
// Local prefix scan of triangle counts

use bevy::render::{
	render_resource::{BindGroupLayout, Buffer, CommandEncoder, ComputePassDescriptor, ComputePipeline},
	renderer::RenderDevice,
};

use crate::pipeline::proc::bind_groups::create_bind_group;

/// Stage 2: Local prefix scan of triangle counts
pub fn stage_prefix_local(
	device: &RenderDevice,
	layout: &BindGroupLayout,
	pipeline: &ComputePipeline,
	block_count: u32,
	tri_counts: &Buffer,
	tri_offset: &Buffer,
	block_sums: &Buffer,
	encoder: &mut CommandEncoder,
) {
	let bind = create_bind_group(
		device,
		"mc_prefix_local_bind",
		layout,
		&[tri_counts, tri_offset, block_sums],
	);

	let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
	pass.set_pipeline(pipeline);
	pass.set_bind_group(0, &bind, &[]);
	pass.dispatch_workgroups(block_count, 1, 1);
}

