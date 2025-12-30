use bevy::{
	prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LeafMaterial {
	#[uniform(0)]
	pub base_color: Vec4, // HSL or RGB in a vec4
}

impl Material for LeafMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/leaf_material.wgsl".into()
	}

	// Enable alpha blending for transparency
	// This allows the leaf shape alpha to create see-through areas
	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Blend
	}
}
