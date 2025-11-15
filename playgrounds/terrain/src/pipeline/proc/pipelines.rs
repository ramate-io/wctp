// =================================================================================================
// PIPELINE CREATION
// =================================================================================================
// Utilities for loading and compiling compute pipelines

use bevy::{
	prelude::*,
	render::render_resource::*,
};

/// Load and compile a compute pipeline from WGSL source.
/// Blocks until the pipeline is ready.
pub fn load_pipeline(
	pipeline_cache: &mut PipelineCache,
	shaders: &mut Assets<Shader>,
	shader_src: &'static str,
	shader_path: &'static str,
	entry: &'static str,
	layout: &BindGroupLayout,
) -> ComputePipeline {
	use std::borrow::Cow;

	let shader = shaders.add(Shader::from_wgsl(shader_src, shader_path));
	let pipeline_descriptor = ComputePipelineDescriptor {
		label: Some(Cow::Owned(format!("compute_pipeline_{}", entry))),
		layout: vec![layout.clone()],
		shader,
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

