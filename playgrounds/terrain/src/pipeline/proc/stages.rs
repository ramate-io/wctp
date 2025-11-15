// =================================================================================================
// PIPELINE STAGES
// =================================================================================================
// Individual compute pass implementations for the Marching Cubes pipeline

use bevy::{
	prelude::UVec3,
	render::{
		render_resource::{BindGroupLayout, Buffer, CommandEncoder, ComputePassDescriptor, ComputePipeline},
		renderer::RenderDevice,
	},
};

use super::bind_groups::create_bind_group;

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

/// Stage 3: Block-level prefix scan
pub fn stage_prefix_block(
	device: &RenderDevice,
	layout: &BindGroupLayout,
	pipeline: &ComputePipeline,
	block_sums: &Buffer,
	block_prefix: &Buffer,
	encoder: &mut CommandEncoder,
) {
	let bind = create_bind_group(
		device,
		"mc_prefix_block_bind",
		layout,
		&[block_sums, block_prefix],
	);

	let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
	pass.set_pipeline(pipeline);
	pass.set_bind_group(0, &bind, &[]);
	pass.dispatch_workgroups(1, 1, 1);
}

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

