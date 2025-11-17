// =================================================================================================
// STAGE 5: MESH
// =================================================================================================
// Generate mesh vertices, normals, and UVs
//
// WGSL binding contract:
//   @group(0) @binding(0) var<uniform> sampling : Sampling3D;
//   @group(0) @binding(1) var<storage, read> cube_index : array<u32>;
//   @group(0) @binding(2) var<storage, read> tri_offset : array<u32>;
//   @group(0) @binding(3) var<storage, read_write> out_positions : array<vec3<f32>>;
//   @group(0) @binding(4) var<storage, read_write> out_normals   : array<vec3<f32>>;
//   @group(0) @binding(5) var<storage, read_write> out_uvs       : array<vec2<f32>>;
//   @group(0) @binding(6) var<uniform> terrain_config : TerrainConfig;
//   @group(0) @binding(7) var<uniform> bounds : Bounds;
//   @group(0) @binding(8) var<uniform> seed : i32;

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

use crate::pipeline::proc::bind_groups::{
	create_bind_group, create_storage_layout_entry, create_uniform_layout_entry,
};

/// Stage for generating mesh vertices, normals, and UVs.
pub struct MeshStage;

impl MeshStage {
	/// Create the bind group layout for the mesh stage.
	pub fn create_layout(device: &RenderDevice) -> BindGroupLayout {
		device.create_bind_group_layout(
			Some("mc_mesh_layout"),
			&[
				create_uniform_layout_entry(0),        // sampling
				create_storage_layout_entry(1, true),  // cube_index (read)
				create_storage_layout_entry(2, true),  // tri_offset (read)
				create_storage_layout_entry(3, false), // out_positions
				create_storage_layout_entry(4, false), // out_normals
				create_storage_layout_entry(5, false), // out_uvs
				create_uniform_layout_entry(6),        // terrain_config
				create_uniform_layout_entry(7),        // bounds
				create_uniform_layout_entry(8),        // seed
			],
		)
	}

	/// Execute the mesh stage using pipeline ID from resource.
	pub fn execute(
		device: &RenderDevice,
		pipeline_cache: &PipelineCache,
		layout: &BindGroupLayout,
		pipeline_id: CachedComputePipelineId,
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
		let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) else {
			log::warn!("Mesh pipeline not ready yet");
			return;
		};

		let bind = create_bind_group(
			device,
			"mc_mesh_bind",
			layout,
			&[
				sampling_buf,
				cube_index,
				tri_offset,
				out_pos,
				out_normals,
				out_uvs,
				cfg_buf,
				bounds_buf,
				seed_buf,
			],
		);

		let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
		pass.set_pipeline(pipeline);
		pass.set_bind_group(0, &bind, &[]);
		pass.dispatch_workgroups(dispatch.x, dispatch.y, dispatch.z);
	}
}
