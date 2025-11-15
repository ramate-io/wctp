// =================================================================================================
// GPU-REPRESENTABLE STRUCTS
// =================================================================================================
// Types that can be safely transferred to/from GPU using bytemuck

use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Bounds {
	pub enabled: u32,
	pub min: Vec2,
	pub max: Vec2,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Sampling3D {
	pub chunk_origin: Vec3,
	pub chunk_size: Vec3,
	pub resolution: UVec3,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TerrainConfigGpu {
	pub seed: u32,
	pub base_resolution: u32,
	pub height_scale: f32,
	pub use_volumetric: u32,
}

impl From<&crate::terrain::TerrainConfig> for TerrainConfigGpu {
	fn from(c: &crate::terrain::TerrainConfig) -> Self {
		Self {
			seed: c.seed,
			base_resolution: c.base_resolution as u32,
			height_scale: c.height_scale,
			use_volumetric: c.use_volumetric as u32,
		}
	}
}

// =================================================================================================
// CPU-SIDE RESULT (RETURNED BY compute())
// =================================================================================================

pub struct GpuMeshData {
	pub positions: Vec<[f32; 3]>,
	pub normals: Vec<[f32; 3]>,
	pub uvs: Vec<[f32; 2]>,
}

