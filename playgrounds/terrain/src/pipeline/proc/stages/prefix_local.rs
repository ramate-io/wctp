// =================================================================================================
// STAGE 2: PREFIX LOCAL
// =================================================================================================
// Local prefix scan of triangle counts
//
// WGSL binding contract:
//   @group(0) @binding(0) var<storage, read>       tri_counts : array<u32>;
//   @group(0) @binding(1) var<storage, read_write> tri_offset : array<u32>;
//   @group(0) @binding(2) var<storage, read_write> block_sums : array<u32>;

use bevy::render::{
	render_resource::{
		BindGroupLayout, Buffer, CommandEncoder, ComputePassDescriptor,
	},
	renderer::RenderDevice,
};

use crate::pipeline::proc::bind_groups::{create_bind_group, create_storage_layout_entry};
use crate::pipeline::proc::pipelines_resource::MarchingCubesPipelines;

/// Stage for local prefix scan of triangle counts.
pub struct PrefixLocalStage;

impl PrefixLocalStage {
	/// Create the bind group layout for the prefix_local stage.
	pub fn create_layout(device: &RenderDevice) -> BindGroupLayout {
		device.create_bind_group_layout(
			Some("mc_prefix_local_layout"),
			&[
				create_storage_layout_entry(0, true),  // tri_counts (read)
				create_storage_layout_entry(1, false), // tri_offset (write)
				create_storage_layout_entry(2, false), // block_sums (write)
			],
		)
	}

	/// Execute the prefix_local stage using pipeline from resource.
	pub fn execute(
		device: &RenderDevice,
		pipelines: &MarchingCubesPipelines,
		layout: &BindGroupLayout,
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
		pass.set_pipeline(&pipelines.prefix_local);
		pass.set_bind_group(0, &bind, &[]);
		pass.dispatch_workgroups(block_count, 1, 1);
	}
}
