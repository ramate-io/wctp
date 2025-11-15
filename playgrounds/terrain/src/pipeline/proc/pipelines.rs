// =================================================================================================
// PIPELINE CREATION
// =================================================================================================
// Utilities for loading and compiling compute pipelines
//
// Note: Shaders are loaded via asset paths to support Bevy's import syntax (#import).
// The asset server processes imports before compilation, which is required for
// naga_oil import resolution.

use bevy::{prelude::*, render::render_resource::*};

/// Load and compile a compute pipeline from an asset path.
/// The shader must be loaded through Bevy's asset system to support imports.
/// Blocks until both the shader is loaded and the pipeline is ready.
pub fn load_pipeline(
	pipeline_cache: &mut PipelineCache,
	asset_server: &AssetServer,
	shaders: &Assets<Shader>,
	shader_path: &str,
	entry: &'static str,
	layout: &BindGroupLayout,
) -> ComputePipeline {
	use std::borrow::Cow;

	// Load shader via asset server - this allows Bevy's shader processor
	// to resolve #import statements (e.g., #import proc::marching_cubes)
	let shader_handle: Handle<Shader> = asset_server.load(shader_path);

	// Wait for the shader to be loaded and processed by the asset system
	// The shader processor needs to resolve imports before we can use it
	loop {
		if let Some(_shader) = shaders.get(&shader_handle) {
			// Shader is loaded, now create pipeline
			let pipeline_descriptor = ComputePipelineDescriptor {
				label: Some(Cow::Owned(format!("compute_pipeline_{}", entry))),
				layout: vec![layout.clone()],
				shader: shader_handle.clone(),
				shader_defs: vec![],
				entry_point: Cow::Borrowed(entry),
				push_constant_ranges: vec![],
				zero_initialize_workgroup_memory: false,
			};

			// Queue the pipeline for compilation
			let pipeline_id = pipeline_cache.queue_compute_pipeline(pipeline_descriptor);

			// Process the pipeline cache until the pipeline is ready
			loop {
				pipeline_cache.process_queue();
				if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) {
					return pipeline.clone();
				}
				std::thread::yield_now();
			}
		}
		std::thread::yield_now();
	}
}
