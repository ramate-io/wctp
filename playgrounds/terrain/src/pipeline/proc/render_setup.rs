// =================================================================================================
// RENDER APP SETUP
// =================================================================================================
// Systems that run in RenderApp to initialize GPU pipelines.
// This is the correct Bevy 0.17 way to create compute pipelines.

use crate::pipeline::proc::pipelines_resource::MarchingCubesPipelines;
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

/// System that runs in RenderApp to create all compute pipelines once at startup.
/// This is the only place where PipelineCache should be accessed.
pub fn setup_marching_cubes_pipelines(
	mut commands: Commands,
	device: Res<bevy::render::renderer::RenderDevice>,
	pipeline_cache: ResMut<PipelineCache>,
	asset_server: Res<AssetServer>,
) {
	// Create bind group layouts for each stage
	let classify_layout = ClassifyStage::create_layout(&device);
	let prefix_local_layout = PrefixLocalStage::create_layout(&device);
	let prefix_block_layout = PrefixBlockStage::create_layout(&device);
	let prefix_add_layout = PrefixAddStage::create_layout(&device);
	let mesh_layout = MeshStage::create_layout(&device);

	// Queue all compute pipelines for compilation
	let classify_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_classify_pipeline".into()),
		layout: vec![classify_layout.clone()],
		shader: asset_server.load("proc/classify_voxels.wgsl"),
		entry_point: "main".into(),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	let prefix_local_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_prefix_local_pipeline".into()),
		layout: vec![prefix_local_layout.clone()],
		shader: asset_server.load("proc/prefix_scan_local.wgsl"),
		entry_point: "main".into(),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	let prefix_block_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_prefix_block_pipeline".into()),
		layout: vec![prefix_block_layout.clone()],
		shader: asset_server.load("proc/block_prefix.wgsl"),
		entry_point: "main".into(),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	let prefix_add_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_prefix_add_pipeline".into()),
		layout: vec![prefix_add_layout.clone()],
		shader: asset_server.load("proc/block_sum.wgsl"),
		entry_point: "main".into(),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	let mesh_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		label: Some("mc_mesh_pipeline".into()),
		layout: vec![mesh_layout.clone()],
		shader: asset_server.load("proc/compute_mesh.wgsl"),
		entry_point: "compute_mesh".into(),
		shader_defs: vec![],
		push_constant_ranges: vec![],
		zero_initialize_workgroup_memory: false,
	});

	// Store pipeline IDs and layouts in a resource
	commands.insert_resource(crate::pipeline::proc::pipelines_resource::MarchingCubesPipelines {
		classify: classify_id,
		prefix_local: prefix_local_id,
		prefix_block: prefix_block_id,
		prefix_add: prefix_add_id,
		mesh: mesh_id,
	});

	// Also store layouts for bind group creation
	commands.insert_resource(MarchingCubesLayouts {
		classify: classify_layout,
		prefix_local: prefix_local_layout,
		prefix_block: prefix_block_layout,
		prefix_add: prefix_add_layout,
		mesh: mesh_layout,
	});
}

/// Extract pipelines from RenderApp to MainApp
pub fn extract_pipelines_to_main_world(
	mut commands: Commands,
	render_pipelines: Extract<Option<Res<MarchingCubesPipelines>>>,
) {
	if let Some(pipelines) = render_pipelines.as_ref() {
		// Clone the inner value from Res<T> (Res<T> derefs to &T)
		commands.insert_resource((**pipelines).clone());
	}
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
