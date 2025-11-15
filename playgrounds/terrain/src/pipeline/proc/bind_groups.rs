// =================================================================================================
// BIND GROUP HELPERS
// =================================================================================================
// Utilities for creating bind group layouts and bind groups

use bevy::render::{render_resource::*, renderer::RenderDevice};

/// Create a bind group layout entry for a uniform buffer.
pub fn create_uniform_layout_entry(binding: u32) -> BindGroupLayoutEntry {
	BindGroupLayoutEntry {
		binding,
		visibility: ShaderStages::COMPUTE,
		ty: BindingType::Buffer {
			ty: BufferBindingType::Uniform,
			has_dynamic_offset: false,
			min_binding_size: None,
		},
		count: None,
	}
}

/// Create a bind group layout entry for a storage buffer.
pub fn create_storage_layout_entry(binding: u32, read_only: bool) -> BindGroupLayoutEntry {
	BindGroupLayoutEntry {
		binding,
		visibility: ShaderStages::COMPUTE,
		ty: BindingType::Buffer {
			ty: BufferBindingType::Storage { read_only },
			has_dynamic_offset: false,
			min_binding_size: None,
		},
		count: None,
	}
}

/// Create a bind group entry from a buffer.
fn create_buffer_entry(binding: u32, buffer: &Buffer) -> BindGroupEntry {
	BindGroupEntry {
		binding,
		resource: BindingResource::Buffer(BufferBinding { buffer, offset: 0, size: None }),
	}
}

/// Create a bind group from a list of buffers.
/// The buffers are bound in order (binding 0, 1, 2, ...).
pub fn create_bind_group(
	device: &RenderDevice,
	label: &str,
	layout: &BindGroupLayout,
	buffers: &[&Buffer],
) -> BindGroup {
	let entries: Vec<BindGroupEntry> = buffers
		.iter()
		.enumerate()
		.map(|(i, buffer)| create_buffer_entry(i as u32, buffer))
		.collect();
	device.create_bind_group(Some(label), layout, &entries)
}
