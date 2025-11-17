// =================================================================================================
// STAGE 3: PREFIX BLOCK
// =================================================================================================
// Block-level prefix scan
//
// WGSL binding contract:
//   @group(0) @binding(0) var<storage, read_write> out_prefix : array<u32>;
//   @group(0) @binding(1) var<storage, read> block_prefix : array<u32>;

use bevy::render::{
	render_resource::{BindGroupLayout, Buffer, CommandEncoder, ComputePassDescriptor},
	renderer::RenderDevice,
};

use crate::pipeline::proc::bind_groups::{create_bind_group, create_storage_layout_entry};
use crate::pipeline::proc::pipelines_resource::MarchingCubesPipelines;

/// Stage for block-level prefix scan.
pub struct PrefixBlockStage;

impl PrefixBlockStage {
	/// Create the bind group layout for the prefix_block stage.
	pub fn create_layout(device: &RenderDevice) -> BindGroupLayout {
		device.create_bind_group_layout(
			Some("mc_prefix_block_layout"),
			&[
				create_storage_layout_entry(0, false), // block_sums (read_write)
				create_storage_layout_entry(1, true),  // block_prefix (read-only)
			],
		)
	}

	/// Execute the prefix_block stage using pipeline from resource.
	pub fn execute(
		device: &RenderDevice,
		pipelines: &MarchingCubesPipelines,
		layout: &BindGroupLayout,
		block_sums: &Buffer,
		block_prefix: &Buffer,
		encoder: &mut CommandEncoder,
	) {
		let bind =
			create_bind_group(device, "mc_prefix_block_bind", layout, &[block_sums, block_prefix]);

		let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
		pass.set_pipeline(&pipelines.prefix_block);
		pass.set_bind_group(0, &bind, &[]);
		pass.dispatch_workgroups(1, 1, 1);
	}
}
