// =================================================================================================
// STAGE 5: MESH
// =================================================================================================
// Generate mesh vertices, normals, and UVs

use bevy::{
	prelude::UVec3,
	render::{
		render_resource::{BindGroupLayout, Buffer, CommandEncoder, ComputePassDescriptor, ComputePipeline},
		renderer::RenderDevice,
	},
};

use crate::pipeline::proc::bind_groups::create_bind_group;

/// Stage 5: Generate mesh vertices, normals, and UVs
pub fn stage_mesh(
	device: &RenderDevice,
	layout: &BindGroupLayout,
	pipeline: &ComputePipeline,
	dispatch: UVec3,
	sampling_buf: &Buffer,
	cube_index: &Buffer,
	tri_offset: &Buffer,
	out_pos: &Buffer,
	out_normals: &Buffer,
	out_uvs: &Buffer,
	cfg_buf: &Buffer,
	bounds_buf: &Buffer,
	seed_buf: &Buffer,
	encoder: &mut CommandEncoder,
) {
	let bind = create_bind_group(
		device,
		"mc_mesh_bind",
		layout,
		&[
			sampling_buf, cube_index, tri_offset, out_pos, out_normals, out_uvs, cfg_buf,
			bounds_buf, seed_buf,
		],
	);

	let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
	pass.set_pipeline(pipeline);
	pass.set_bind_group(0, &bind, &[]);
	pass.dispatch_workgroups(dispatch.x, dispatch.y, dispatch.z);
}

