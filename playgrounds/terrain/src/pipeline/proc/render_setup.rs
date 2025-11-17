// =================================================================================================
// RENDER APP SETUP
// =================================================================================================
// Systems that run in RenderApp to initialize GPU pipelines.
// This is the correct Bevy 0.17 way to create compute pipelines.

use crate::pipeline::proc::pipelines_resource::{
	MarchingCubesPipelineIds, MarchingCubesPipelines, MarchingCubesShaderHandles,
};
use crate::pipeline::proc::stages::{
	ClassifyStage, MeshStage, PrefixAddStage, PrefixBlockStage, PrefixLocalStage,
};
use bevy::{
	prelude::*,
	render::{
		render_resource::{ComputePipelineDescriptor, PipelineCache},
		Extract,
	},
};

/// Stage 1: Queue all compute pipelines for compilation.
/// This runs once at startup and queues pipelines for asynchronous compilation.
pub fn queue_marching_cubes_pipelines(
	mut commands: Commands,
	device: Res<bevy::render::renderer::RenderDevice>,
	pipeline_cache: ResMut<PipelineCache>,
	asset_server: Res<AssetServer>,
	ids: Option<Res<MarchingCubesPipelineIds>>,
) {
	// If we've already queued pipelines, skip
	if ids.is_some() {
		return;
	}
	// Create bind group layouts for each stage
	let classify_layout = ClassifyStage::create_layout(&device);
	let prefix_local_layout = PrefixLocalStage::create_layout(&device);
	let prefix_block_layout = PrefixBlockStage::create_layout(&device);
	let prefix_add_layout = PrefixAddStage::create_layout(&device);
	let mesh_layout = MeshStage::create_layout(&device);

	// Load shader assets first
	let classify_shader: Handle<Shader> = asset_server.load("proc/classify_voxels.wgsl");
	let prefix_local_shader: Handle<Shader> = asset_server.load("proc/prefix_scan_local.wgsl");
	let prefix_block_shader: Handle<Shader> = asset_server.load("proc/block_prefix.wgsl");
	let prefix_add_shader: Handle<Shader> = asset_server.load("proc/block_sum.wgsl");
	let mesh_shader: Handle<Shader> = asset_server.load("proc/compute_mesh.wgsl");

	// Queue all compute pipelines for compilation (asynchronous)
	let classify_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_classify_pipeline".into()),
		layout: vec![classify_layout.clone()],
		shader: classify_shader.clone(),
		entry_point: Some("main".into()),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	let prefix_local_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_prefix_local_pipeline".into()),
		layout: vec![prefix_local_layout.clone()],
		shader: prefix_local_shader.clone(),
		entry_point: Some("main".into()),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	let prefix_block_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_prefix_block_pipeline".into()),
		layout: vec![prefix_block_layout.clone()],
		shader: prefix_block_shader.clone(),
		entry_point: Some("main".into()),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	let prefix_add_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_prefix_add_pipeline".into()),
		layout: vec![prefix_add_layout.clone()],
		shader: prefix_add_shader.clone(),
		entry_point: Some("main".into()),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	let mesh_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_mesh_pipeline".into()),
		layout: vec![mesh_layout.clone()],
		shader: mesh_shader.clone(),
		entry_point: Some("compute_mesh".into()),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	// Store pipeline IDs for polling
	commands.insert_resource(MarchingCubesPipelineIds {
		classify: classify_id,
		prefix_local: prefix_local_id,
		prefix_block: prefix_block_id,
		prefix_add: prefix_add_id,
		mesh: mesh_id,
	});

	// Store shader handles for checking loading state
	commands.insert_resource(MarchingCubesShaderHandles {
		classify: classify_shader,
		prefix_local: prefix_local_shader,
		prefix_block: prefix_block_shader,
		prefix_add: prefix_add_shader,
		mesh: mesh_shader,
	});

	// Store layouts for bind group creation
	commands.insert_resource(MarchingCubesLayouts {
		classify: classify_layout,
		prefix_local: prefix_local_layout,
		prefix_block: prefix_block_layout,
		prefix_add: prefix_add_layout,
		mesh: mesh_layout,
	});
}

/// Stage 2: Poll for pipeline readiness and store actual pipelines when ready.
/// This runs every frame until all pipelines are compiled, then stores them.
pub fn init_marching_cubes_pipelines(
	mut commands: Commands,
	mut pipeline_cache: ResMut<PipelineCache>,
	ids: Option<Res<MarchingCubesPipelineIds>>,
	shader_handles: Option<Res<MarchingCubesShaderHandles>>,
	shaders: Res<Assets<Shader>>,
) {
	// If IDs resource doesn't exist, pipelines are already initialized
	let Some(ids) = ids else {
		return;
	};
	let Some(shader_handles) = shader_handles else {
		return;
	};

	// Check if all shaders are loaded before processing the queue
	let all_shaders_loaded = shaders.get(&shader_handles.classify).is_some()
		&& shaders.get(&shader_handles.prefix_local).is_some()
		&& shaders.get(&shader_handles.prefix_block).is_some()
		&& shaders.get(&shader_handles.prefix_add).is_some()
		&& shaders.get(&shader_handles.mesh).is_some();

	if !all_shaders_loaded {
		log::debug!("Waiting for shaders to load...");
		return; // Wait for shaders to load
	}

	log::debug!("All shaders loaded, processing pipeline queue");

	// Process the pipeline queue to advance compilation
	// This must be called to actually compile the queued pipelines
	pipeline_cache.process_queue();

	// Check pipeline state for debugging - this will tell us if shaders are loaded
	let classify_state = pipeline_cache.get_compute_pipeline_state(ids.classify);
	match classify_state {
		bevy::render::render_resource::CachedPipelineState::Queued => {
			log::debug!("Classify pipeline still queued (shader may not be loaded yet)");
		}
		bevy::render::render_resource::CachedPipelineState::Creating(_) => {
			log::debug!("Classify pipeline is being created");
		}
		bevy::render::render_resource::CachedPipelineState::Err(err) => {
			log::warn!("Classify pipeline error: {:?}", err);
		}
		bevy::render::render_resource::CachedPipelineState::Ok(_) => {
			log::debug!("Classify pipeline is ready!");
		}
	}

	// Try to get all pipelines from the cache
	let Some(classify) = pipeline_cache.get_compute_pipeline(ids.classify) else {
		warn!("Classify pipeline not ready yet (state: {:?})", classify_state);
		return; // Not ready yet, try again next frame
	};
	let Some(prefix_local) = pipeline_cache.get_compute_pipeline(ids.prefix_local) else {
		warn!("Prefix local pipeline not ready yet");
		return;
	};
	let Some(prefix_block) = pipeline_cache.get_compute_pipeline(ids.prefix_block) else {
		warn!("Prefix block pipeline not ready yet");
		return;
	};
	let Some(prefix_add) = pipeline_cache.get_compute_pipeline(ids.prefix_add) else {
		warn!("Prefix add pipeline not ready yet");
		return;
	};
	let Some(mesh) = pipeline_cache.get_compute_pipeline(ids.mesh) else {
		warn!("Mesh pipeline not ready yet");
		return;
	};

	// All pipelines are ready! Store them and remove the temporary resources
	commands.insert_resource(MarchingCubesPipelines {
		classify: classify.clone(),
		prefix_local: prefix_local.clone(),
		prefix_block: prefix_block.clone(),
		prefix_add: prefix_add.clone(),
		mesh: mesh.clone(),
	});

	// Remove the temporary resources since we no longer need them
	commands.remove_resource::<MarchingCubesPipelineIds>();
	commands.remove_resource::<MarchingCubesShaderHandles>();

	log::info!("Marching Cubes GPU pipelines ready");
}

/// Extract pipelines from RenderApp to MainApp
/// Note: ComputePipeline is not Clone, so we can't extract it directly.
/// Instead, we'll recreate layouts in MainApp when needed.
/// The pipelines resource stays in RenderApp and is accessed via Extract.
pub fn extract_pipelines_to_main_world(
	_commands: Commands,
	_render_pipelines: Extract<Option<Res<MarchingCubesPipelines>>>,
) {
	// Pipelines are accessed directly from RenderApp via Extract when needed
	// Since ComputePipeline is not Clone, we can't extract it to MainApp
	// The chunk_manager will need to access pipelines differently
}

/// Resource containing all bind group layouts for the marching cubes pipeline.
/// Note: BindGroupLayout is not Clone, so layouts are recreated in the main world when needed.
/// This resource is only used in RenderApp.
#[derive(Resource)]
pub struct MarchingCubesLayouts {
	pub classify: bevy::render::render_resource::BindGroupLayout,
	pub prefix_local: bevy::render::render_resource::BindGroupLayout,
	pub prefix_block: bevy::render::render_resource::BindGroupLayout,
	pub prefix_add: bevy::render::render_resource::BindGroupLayout,
	pub mesh: bevy::render::render_resource::BindGroupLayout,
}

/// Helper to create layouts in the main world (since BindGroupLayout is not Clone)
pub fn create_layouts_in_main_world(
	device: &bevy::render::renderer::RenderDevice,
) -> MarchingCubesLayouts {
	use crate::pipeline::proc::stages::{
		ClassifyStage, MeshStage, PrefixAddStage, PrefixBlockStage, PrefixLocalStage,
	};
	MarchingCubesLayouts {
		classify: ClassifyStage::create_layout(device),
		prefix_local: PrefixLocalStage::create_layout(device),
		prefix_block: PrefixBlockStage::create_layout(device),
		prefix_add: PrefixAddStage::create_layout(device),
		mesh: MeshStage::create_layout(device),
	}
}
