// =================================================================================================
// STAGE 4: PREFIX ADD
// =================================================================================================
// Add block prefixes to local offsets
//
// WGSL binding contract:
//   @group(0) @binding(0) var<storage, read_write> tri_offset : array<u32>;
//   @group(0) @binding(1) var<storage, read>       block_prefix : array<u32>;

use bevy::render::{
	render_resource::{
		BindGroupLayout, Buffer, CachedComputePipelineId, CommandEncoder,
		ComputePassDescriptor, PipelineCache,
	},
	renderer::RenderDevice,
};

use crate::pipeline::proc::bind_groups::{create_bind_group, create_storage_layout_entry};

/// Stage for adding block prefixes to local offsets.
pub struct PrefixAddStage;

impl PrefixAddStage {
	/// Create the bind group layout for the prefix_add stage.
	pub fn create_layout(device: &RenderDevice) -> BindGroupLayout {
		device.create_bind_group_layout(
			Some("mc_prefix_add_layout"),
			&[
				create_storage_layout_entry(0, false), // tri_offset
				create_storage_layout_entry(1, true),  // block_prefix (read)
			],
		)
	}

	/// Execute the prefix_add stage using pipeline ID from resource.
	pub fn execute(
		device: &RenderDevice,
		pipeline_cache: &PipelineCache,
		layout: &BindGroupLayout,
		pipeline_id: CachedComputePipelineId,
		block_count: u32,
		tri_offset: &Buffer,
		block_prefix: &Buffer,
		encoder: &mut CommandEncoder,
	) {
		let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) else {
			log::warn!("Prefix add pipeline not ready yet");
			return;
		};

		let bind = create_bind_group(device, "mc_prefix_add_bind", layout, &[tri_offset, block_prefix]);

		let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
		pass.set_pipeline(pipeline);
		pass.set_bind_group(0, &bind, &[]);
		pass.dispatch_workgroups(block_count, 1, 1);
	}
}
