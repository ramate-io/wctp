use bevy::{
	prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct EdgeMaterial {}

impl Material for EdgeMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/edge_material.wgsl".into()
	}

	fn vertex_shader() -> ShaderRef {
		"shaders/edge_material.wgsl".into()
	}
}
