// =================================================================================================
// BUFFER HELPERS
// =================================================================================================
// Utilities for creating GPU buffers and reading data back to CPU

use bevy::render::{
	render_resource::*,
	renderer::{RenderDevice, RenderQueue},
};
use bytemuck::Pod;

/// Create a new storage buffer with the specified size in bytes.
pub fn new_storage(device: &RenderDevice, size_bytes: usize) -> Buffer {
	device.create_buffer(&BufferDescriptor {
		label: None,
		size: size_bytes as u64,
		usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
		mapped_at_creation: false,
	})
}

/// Create a new uniform buffer from a Pod type.
pub fn new_uniform<T: Pod>(device: &RenderDevice, v: &T) -> Buffer {
	device.create_buffer_with_data(&BufferInitDescriptor {
		label: None,
		contents: bytemuck::bytes_of(v),
		usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
	})
}

/// Read a vector of Pod types from a GPU buffer to CPU.
pub fn read_vec<T: Pod>(
	device: &RenderDevice,
	queue: &RenderQueue,
	src: &Buffer,
	count: usize,
) -> Vec<T> {
	let size = (count * std::mem::size_of::<T>()) as u64;

	let staging = device.create_buffer(&BufferDescriptor {
		label: None,
		size,
		usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
		mapped_at_creation: false,
	});

	let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
	encoder.copy_buffer_to_buffer(src, 0, &staging, 0, size);
	queue.submit(Some(encoder.finish()));

	let slice = staging.slice(..);
	slice.map_async(MapMode::Read, |_| {});
	device.poll(Maintain::Wait);

	let range = slice.get_mapped_range();
	let result: Vec<T> = bytemuck::cast_slice(&range).to_vec();
	drop(range);
	staging.unmap();

	result
}

/// Read a single u32 from a GPU buffer at the specified index.
pub fn read_u32(device: &RenderDevice, queue: &RenderQueue, src: &Buffer, idx: u32) -> u32 {
	let staging = device.create_buffer(&BufferDescriptor {
		label: None,
		size: 4,
		usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
		mapped_at_creation: false,
	});

	let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
	encoder.copy_buffer_to_buffer(src, (idx * 4) as u64, &staging, 0, 4);
	queue.submit(Some(encoder.finish()));

	let slice = staging.slice(..);
	slice.map_async(MapMode::Read, |_| {});
	device.poll(Maintain::Wait);

	let range = slice.get_mapped_range();
	let v = u32::from_le_bytes(range[0..4].try_into().unwrap());
	drop(range);
	staging.unmap();
	v
}
