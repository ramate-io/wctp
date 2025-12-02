use bevy::{
	prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
	#[uniform(0)]
	color: LinearRgba,
	#[texture(1)]
	#[sampler(2)]
	color_texture: Option<Handle<Image>>,
	alpha_mode: AlphaMode,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/custom_material.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		self.alpha_mode
	}
}
