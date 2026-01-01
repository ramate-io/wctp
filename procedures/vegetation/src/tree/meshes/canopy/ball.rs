use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use noise::Perlin;
use render_item::{
	mesh::{IdentifiedMesh, MeshBuilder, MeshId},
	NormalizeChunk,
};
use std::f32::consts::PI;

/// Configuration for a noisy sphere/ball
/// All balls work in unit space (0-1) and are transformed later
#[derive(Debug, Clone)]
pub struct NoisyBallConfig {
	/// Seed for noise generation
	pub seed: u32,
	/// Base radius of the sphere (in unit space, typically 0.5)
	pub radius: f32,
	/// Noise amplitude for surface variation
	/// Higher values create more pronounced surface bumps
	pub noise_amplitude: f32,
	/// Noise frequency for surface variation
	/// Higher values create finer, more detailed noise patterns
	pub noise_frequency: f32,
	/// Number of noise octaves for fractal detail
	/// More octaves = more detailed but potentially slower
	pub noise_octaves: u32,
}

impl Default for NoisyBallConfig {
	fn default() -> Self {
		Self { seed: 0, radius: 0.5, noise_amplitude: 0.1, noise_frequency: 3.0, noise_octaves: 3 }
	}
}

/// Noisy sphere: a sphere with Perlin noise perturbation for organic surface variation
#[derive(Debug, Clone)]
pub struct NoisyBall {
	config: NoisyBallConfig,
	noise: Perlin,
}

impl NoisyBall {
	pub fn new(config: NoisyBallConfig) -> Self {
		let noise = Perlin::new(config.seed);
		Self { config, noise }
	}
}

/*impl Sdf for NoisyBall {
	/// Distance function for a noisy sphere
	/// The sphere is centered at the origin with configurable radius
	/// Perlin noise is used to perturb the surface for organic variation
	fn distance(&self, p: Vec3) -> f32 {
		// Distance from center
		let dist_from_center = p.length();

		// Base sphere distance (negative inside, positive outside)
		let mut dist = dist_from_center - self.config.radius;

		// Add noise perturbation for surface variation
		// Sample noise at the point's position, scaled by frequency
		let noise_value = self.noise.get([
			p.x as f64 * self.config.noise_frequency as f64,
			p.y as f64 * self.config.noise_frequency as f64,
			p.z as f64 * self.config.noise_frequency as f64,
		]) as f32;

		// Apply noise amplitude to the distance
		// This creates bumps and indentations on the sphere surface
		dist += noise_value * self.config.noise_amplitude;

		dist
	}
}*/

impl NormalizeChunk for NoisyBall {
	fn normalize_chunk(&self, cascade_chunk: &CascadeChunk) -> CascadeChunk {
		CascadeChunk::unit_3d_center_chunk()
			.with_res_2(cascade_chunk.res_2)
			.with_mu(self.config.noise_amplitude + 0.001)
	}
}

impl IdentifiedMesh for NoisyBall {
	fn id(&self) -> MeshId {
		let debug_string = format!("{:?}", self);
		MeshId::new(debug_string)
	}
}

impl MeshBuilder for NoisyBall {
	fn build_mesh_impl(&self, _cascade_chunk: &CascadeChunk) -> Option<Mesh> {
		// Generate 8 unit disk meshes intersecting at the origin
		let num_disks = 8;
		let radius = 1.0; // Unit disk
		let segments = 32; // Number of segments for the disk

		// Use Fibonacci sphere algorithm for even distribution of directions
		let golden_angle = PI * (3.0 - (5.0_f32).sqrt());

		let mut all_vertices: Vec<[f32; 3]> = Vec::new();
		let mut all_normals: Vec<[f32; 3]> = Vec::new();
		let mut all_uvs: Vec<[f32; 2]> = Vec::new();
		let mut all_indices: Vec<u32> = Vec::new();

		for i in 0..num_disks {
			// Calculate direction using Fibonacci sphere for even distribution
			let theta = golden_angle * i as f32;
			let y = 1.0 - (2.0 * i as f32) / (num_disks as f32 - 1.0); // y goes from 1 to -1
			let radius_at_y = (1.0 - y * y).sqrt(); // Radius at this y level

			// Calculate direction vector (normal of the disc)
			let x = radius_at_y * theta.cos();
			let z = radius_at_y * theta.sin();
			let direction = Vec3::new(x, y, z).normalize();

			// Generate disk geometry in the XY plane (normal pointing along Z)
			let (disk_vertices, disk_normals, disk_uvs, disk_indices) =
				generate_unit_disk(radius, segments);

			// Transform disk to the appropriate orientation
			// Disk's default normal is Vec3::Z, so rotate Z to direction
			let rotation = if direction.abs_diff_eq(Vec3::Z, 1e-4) {
				Quat::IDENTITY
			} else {
				Quat::from_rotation_arc(Vec3::Z, direction)
			};

			// Apply rotation to vertices and normals
			let vertex_offset = all_vertices.len() as u32;
			for (vertex, normal) in disk_vertices.iter().zip(disk_normals.iter()) {
				let v = Vec3::new(vertex[0], vertex[1], vertex[2]);
				let n = Vec3::new(normal[0], normal[1], normal[2]);
				let rotated_v = rotation * v;
				let rotated_n = rotation * n;
				all_vertices.push([rotated_v.x, rotated_v.y, rotated_v.z]);
				all_normals.push([rotated_n.x, rotated_n.y, rotated_n.z]);
			}

			// Add UVs (no transformation needed)
			all_uvs.extend(disk_uvs);

			// Add indices with offset
			all_indices.extend(disk_indices.iter().map(|&idx| idx + vertex_offset));
		}

		// Create the mesh
		let mut mesh = Mesh::new(
			bevy::mesh::PrimitiveTopology::TriangleList,
			bevy::asset::RenderAssetUsages::RENDER_WORLD,
		);
		mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, all_vertices);
		mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, all_normals);
		mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, all_uvs);
		mesh.insert_indices(bevy::mesh::Indices::U32(all_indices));

		Some(mesh)
	}
}

/// Generate a unit disk mesh in the XY plane (normal pointing along +Z)
/// Returns (vertices, normals, uvs, indices)
fn generate_unit_disk(
	radius: f32,
	segments: u32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
	let mut vertices = Vec::new();
	let mut normals = Vec::new();
	let mut uvs = Vec::new();
	let mut indices = Vec::new();

	// Center vertex at origin
	vertices.push([0.0, 0.0, 0.0]);
	normals.push([0.0, 0.0, 1.0]);
	uvs.push([0.5, 0.5]);

	// Generate vertices around the circle
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

	// Generate triangle indices (fan from center)
	for i in 0..segments {
		indices.push(0); // Center vertex
		indices.push(i + 1);
		indices.push(i + 2);
	}

	(vertices, normals, uvs, indices)
}
