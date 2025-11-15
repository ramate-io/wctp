// =================================================================================================
// STAGE 4: PREFIX ADD
// =================================================================================================
// Add block prefixes to local offsets
//
// WGSL binding contract:
//   @group(0) @binding(0) var<storage, read_write> tri_offset : array<u32>;
//   @group(0) @binding(1) var<storage, read>       block_prefix : array<u32>;

use bevy::{
	prelude::{AssetServer, Assets, Shader},
	render::{
		render_resource::{
			BindGroupLayout, Buffer, CommandEncoder, ComputePassDescriptor, ComputePipeline,
			PipelineCache,
		},
		renderer::RenderDevice,
	},
};

use crate::pipeline::proc::bind_groups::{create_bind_group, create_storage_layout_entry};
use crate::pipeline::proc::pipelines::load_pipeline;

pub struct PrefixAddStage {
	pub layout: BindGroupLayout,
	pub pipeline: ComputePipeline,
}

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

	/// Load the compute pipeline for the prefix_add stage.
	pub fn load_pipeline(
		pipeline_cache: &mut PipelineCache,
		asset_server: &AssetServer,
		shaders: &Assets<Shader>,
		layout: &BindGroupLayout,
	) -> ComputePipeline {
		load_pipeline(pipeline_cache, asset_server, shaders, "proc/block_sum.wgsl", "main", layout)
	}

	/// Create a new PrefixAddStage with layout and pipeline.
	pub fn new(
		device: &RenderDevice,
		pipeline_cache: &mut PipelineCache,
		asset_server: &AssetServer,
		shaders: &Assets<Shader>,
	) -> Self {
		let layout = Self::create_layout(device);
		let pipeline = Self::load_pipeline(pipeline_cache, asset_server, shaders, &layout);
		Self { layout, pipeline }
	}

	/// Execute the prefix_add stage.
	pub fn execute(
		&self,
		device: &RenderDevice,
		block_count: u32,
		tri_offset: &Buffer,
		block_prefix: &Buffer,
		encoder: &mut CommandEncoder,
	) {
		let bind = create_bind_group(
			device,
			"mc_prefix_add_bind",
			&self.layout,
			&[tri_offset, block_prefix],
		);

		let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
		pass.set_pipeline(&self.pipeline);
		pass.set_bind_group(0, &bind, &[]);
		pass.dispatch_workgroups(block_count, 1, 1);
	}
}
