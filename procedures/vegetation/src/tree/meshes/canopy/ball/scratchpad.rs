use bevy::prelude::*;
use std::f32::consts::PI;

/// Generate a unit triangle mesh in the XY plane (normal pointing along +Z)
/// Returns (vertices, normals, uvs, indices)
/// size controls the overall scale of the triangle
/// Includes both front and back faces for double-sided rendering
pub fn generate_unit_triangle(
	size: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
	let mut vertices = Vec::new();
	let mut normals = Vec::new();
	let mut uvs = Vec::new();
	let mut indices = Vec::new();

	// Create an equilateral triangle centered at origin
	// Height of equilateral triangle: h = sqrt(3) / 2 * side
	// For a triangle with vertices at distance 'size' from center:
	// We'll use a circumradius approach
	let circumradius = size;

	// Front face vertices (normal pointing +Z)
	// First vertex at top
	vertices.push([0.0, circumradius, 0.0]);
	normals.push([0.0, 0.0, 1.0]);
	uvs.push([0.5, 1.0]);

	// Second vertex at bottom-left
	let angle_1 = 2.0 * PI / 3.0;
	vertices.push([circumradius * angle_1.cos(), circumradius * angle_1.sin(), 0.0]);
	normals.push([0.0, 0.0, 1.0]);
	uvs.push([0.0, 0.0]);

	// Third vertex at bottom-right
	let angle_2 = 4.0 * PI / 3.0;
	vertices.push([circumradius * angle_2.cos(), circumradius * angle_2.sin(), 0.0]);
	normals.push([0.0, 0.0, 1.0]);
	uvs.push([1.0, 0.0]);

	// Front face triangle (counter-clockwise when viewed from +Z)
	indices.push(0);
	indices.push(1);
	indices.push(2);

	// Back face vertices (normal pointing -Z) - same positions, flipped normals
	let front_vertex_count = vertices.len() as u32;
	vertices.push([0.0, circumradius, 0.0]);
	normals.push([0.0, 0.0, -1.0]);
	uvs.push([0.5, 1.0]);

	vertices.push([circumradius * angle_1.cos(), circumradius * angle_1.sin(), 0.0]);
	normals.push([0.0, 0.0, -1.0]);
	uvs.push([0.0, 0.0]);

	vertices.push([circumradius * angle_2.cos(), circumradius * angle_2.sin(), 0.0]);
	normals.push([0.0, 0.0, -1.0]);
	uvs.push([1.0, 0.0]);

	// Back face triangle (clockwise when viewed from +Z, so reversed winding)
	indices.push(front_vertex_count);
	indices.push(front_vertex_count + 2);
	indices.push(front_vertex_count + 1);

	(vertices, normals, uvs, indices)
}

/// Generate a unit rectangle mesh in the XY plane (normal pointing along +Z)
/// Returns (vertices, normals, uvs, indices)
/// size is the half-extent (so total size is 2*size x 2*size)
/// Includes both front and back faces for double-sided rendering
pub fn generate_unit_rectangle(
	size: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
	let mut vertices = Vec::new();
	let mut normals = Vec::new();
	let mut uvs = Vec::new();
	let mut indices = Vec::new();

	// Front face vertices (normal pointing +Z)
	// Vertices in counter-clockwise order when viewed from +Z
	// Bottom-left
	vertices.push([-size, -size, 0.0]);
	normals.push([0.0, 0.0, 1.0]);
	uvs.push([0.0, 0.0]);

	// Bottom-right
	vertices.push([size, -size, 0.0]);
	normals.push([0.0, 0.0, 1.0]);
	uvs.push([1.0, 0.0]);

	// Top-right
	vertices.push([size, size, 0.0]);
	normals.push([0.0, 0.0, 1.0]);
	uvs.push([1.0, 1.0]);

	// Top-left
	vertices.push([-size, size, 0.0]);
	normals.push([0.0, 0.0, 1.0]);
	uvs.push([0.0, 1.0]);

	// Front face triangles
	// Triangle 1: bottom-left, bottom-right, top-right
	indices.push(0);
	indices.push(1);
	indices.push(2);

	// Triangle 2: bottom-left, top-right, top-left
	indices.push(0);
	indices.push(2);
	indices.push(3);

	// Back face vertices (normal pointing -Z) - same positions, flipped normals
	let front_vertex_count = vertices.len() as u32;
	vertices.push([-size, -size, 0.0]);
	normals.push([0.0, 0.0, -1.0]);
	uvs.push([0.0, 0.0]);

	vertices.push([size, -size, 0.0]);
	normals.push([0.0, 0.0, -1.0]);
	uvs.push([1.0, 0.0]);

	vertices.push([size, size, 0.0]);
	normals.push([0.0, 0.0, -1.0]);
	uvs.push([1.0, 1.0]);

	vertices.push([-size, size, 0.0]);
	normals.push([0.0, 0.0, -1.0]);
	uvs.push([0.0, 1.0]);

	// Back face triangles (reversed winding)
	// Triangle 1: bottom-left, top-right, bottom-right
	indices.push(front_vertex_count);
	indices.push(front_vertex_count + 2);
	indices.push(front_vertex_count + 1);

	// Triangle 2: bottom-left, top-left, top-right
	indices.push(front_vertex_count);
	indices.push(front_vertex_count + 3);
	indices.push(front_vertex_count + 2);

	(vertices, normals, uvs, indices)
}

/// Generate a unit disk mesh in the XY plane (normal pointing along +Z)
/// Returns (vertices, normals, uvs, indices)
/// Includes both front and back faces for double-sided rendering
pub fn generate_unit_disk(
	radius: f32,
	segments: u32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
	let mut vertices = Vec::new();
	let mut normals = Vec::new();
	let mut uvs = Vec::new();
	let mut indices = Vec::new();

	// Front face center vertex
	vertices.push([0.0, 0.0, 0.0]);
	normals.push([0.0, 0.0, 1.0]);
	uvs.push([0.5, 0.5]);

	// Generate front face vertices around the circle
	for i in 0..=segments {
		let angle = 2.0 * PI * i as f32 / segments as f32;
		let x = radius * angle.cos();
		let y = radius * angle.sin();
		vertices.push([x, y, 0.0]);
		normals.push([0.0, 0.0, 1.0]);
		// UV coordinates from center (0.5, 0.5) to edge
		let u = 0.5 + 0.5 * angle.cos();
		let v = 0.5 + 0.5 * angle.sin();
		uvs.push([u, v]);
	}

	// Generate front face triangle indices (fan from center, counter-clockwise)
	for i in 0..segments {
		indices.push(0); // Center vertex
		indices.push(i + 1);
		indices.push(i + 2);
	}

	// Back face center vertex
	let front_vertex_count = vertices.len() as u32;
	vertices.push([0.0, 0.0, 0.0]);
	normals.push([0.0, 0.0, -1.0]);
	uvs.push([0.5, 0.5]);

	// Generate back face vertices around the circle (same positions, flipped normals)
	for i in 0..=segments {
		let angle = 2.0 * PI * i as f32 / segments as f32;
		let x = radius * angle.cos();
		let y = radius * angle.sin();
		vertices.push([x, y, 0.0]);
		normals.push([0.0, 0.0, -1.0]);
		// UV coordinates from center (0.5, 0.5) to edge
		let u = 0.5 + 0.5 * angle.cos();
		let v = 0.5 + 0.5 * angle.sin();
		uvs.push([u, v]);
	}

	// Generate back face triangle indices (fan from center, clockwise/reversed)
	for i in 0..segments {
		indices.push(front_vertex_count); // Back center vertex
		indices.push(front_vertex_count + i + 2); // Reversed order
		indices.push(front_vertex_count + i + 1);
	}

	(vertices, normals, uvs, indices)
}
