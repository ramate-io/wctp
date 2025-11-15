// =================================================================================================
// STAGE 1: CLASSIFY
// =================================================================================================
// Classify voxels and compute cube indices
//
// WGSL binding contract:
//   @group(0) @binding(0) var<uniform> sampling : Sampling3D;
//   @group(0) @binding(1) var<uniform> terrain_config : TerrainConfig;
//   @group(0) @binding(2) var<uniform> bounds : Bounds;
//   @group(0) @binding(3) var<uniform> seed : i32;
//   @group(0) @binding(4) var<storage, read_write> cube_index : array<u32>;
//   @group(0) @binding(5) var<storage, read_write> tri_counts : array<u32>;

use bevy::{
	prelude::{AssetServer, Assets, Shader, UVec3},
	render::{
		render_resource::{
			BindGroupLayout, Buffer, CommandEncoder, ComputePassDescriptor, ComputePipeline,
			PipelineCache,
		},
		renderer::RenderDevice,
	},
};

use crate::pipeline::proc::bind_groups::{create_bind_group, create_storage_layout_entry, create_uniform_layout_entry};
use crate::pipeline::proc::pipelines::load_pipeline;

pub struct ClassifyStage {
	pub layout: BindGroupLayout,
	pub pipeline: ComputePipeline,
}

impl ClassifyStage {
	/// Create the bind group layout for the classify stage.
	pub fn create_layout(device: &RenderDevice) -> BindGroupLayout {
		device.create_bind_group_layout(
			Some("mc_classify_layout"),
			&[
				create_uniform_layout_entry(0),        // sampling
				create_uniform_layout_entry(1),        // terrain_config
				create_uniform_layout_entry(2),        // bounds
				create_uniform_layout_entry(3),        // seed
				create_storage_layout_entry(4, false), // cube_index
				create_storage_layout_entry(5, false), // tri_counts
			],
		)
	}

	/// Load the compute pipeline for the classify stage.
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
			"proc/classify_voxels.wgsl",
			"main",
			layout,
		)
	}

	/// Create a new ClassifyStage with layout and pipeline.
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

	/// Execute the classify stage.
	pub fn execute(
		&self,
		device: &RenderDevice,
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
			&self.layout,
			&[sampling_buf, cfg_buf, bounds_buf, seed_buf, cube_index, tri_counts],
		);

		let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
		pass.set_pipeline(&self.pipeline);
		pass.set_bind_group(0, &bind, &[]);
		pass.dispatch_workgroups(dispatch.x, dispatch.y, dispatch.z);
	}
}
