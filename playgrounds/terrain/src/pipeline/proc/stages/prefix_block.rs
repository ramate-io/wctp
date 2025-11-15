// =================================================================================================
// STAGE 3: PREFIX BLOCK
// =================================================================================================
// Block-level prefix scan
//
// WGSL binding contract:
//   @group(0) @binding(0) var<storage, read_write> block_sums : array<u32>;
//   @group(0) @binding(1) var<storage, read_write> block_prefix : array<u32>;

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

pub struct PrefixBlockStage {
	pub layout: BindGroupLayout,
	pub pipeline: ComputePipeline,
}

impl PrefixBlockStage {
	/// Create the bind group layout for the prefix_block stage.
	pub fn create_layout(device: &RenderDevice) -> BindGroupLayout {
		device.create_bind_group_layout(
			Some("mc_prefix_block_layout"),
			&[
				create_storage_layout_entry(0, false), // block_sums
				create_storage_layout_entry(1, false), // block_prefix
			],
		)
	}

	/// Load the compute pipeline for the prefix_block stage.
	pub fn load_pipeline(
		pipeline_cache: &mut PipelineCache,
		asset_server: &AssetServer,
		shaders: &Assets<Shader>,
		layout: &BindGroupLayout,
	) -> ComputePipeline {
		load_pipeline(
			pipeline_cache,
			asset_server,
			shaders,
			"proc/block_prefix.wgsl",
			"main",
			layout,
		)
	}

	/// Create a new PrefixBlockStage with layout and pipeline.
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

	/// Execute the prefix_block stage.
	pub fn execute(
		&self,
		device: &RenderDevice,
		block_sums: &Buffer,
		block_prefix: &Buffer,
		encoder: &mut CommandEncoder,
	) {
		let bind = create_bind_group(
			device,
			"mc_prefix_block_bind",
			&self.layout,
			&[block_sums, block_prefix],
		);

		let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
		pass.set_pipeline(&self.pipeline);
		pass.set_bind_group(0, &bind, &[]);
		pass.dispatch_workgroups(1, 1, 1);
	}
}
