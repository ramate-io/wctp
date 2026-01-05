use crate::complex::{Floor, Partition};
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::{
	mesh::{
		cache::handle::map::HandleMap, handle::MeshHandle, IdentifiedMesh, MeshBuilder,
		MeshDispatch, MeshId,
	},
	NormalizeChunk, RenderItem,
};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// Noisy sphere: a sphere with Perlin noise perturbation for organic surface variation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WallMesh {}

impl WallMesh {
	pub fn new() -> Self {
		Self {}
	}
}

impl NormalizeChunk for WallMesh {
	fn normalize_chunk(&self, cascade_chunk: &CascadeChunk) -> CascadeChunk {
		CascadeChunk::unit_3d_center_chunk().with_res_2(cascade_chunk.res_2)
	}
}

impl IdentifiedMesh for WallMesh {
	fn id(&self) -> MeshId {
		let debug_string = format!("{:?}", self);
		MeshId::new(debug_string)
	}
}

impl MeshBuilder for WallMesh {
	fn build_mesh_impl(&self, _cascade_chunk: &CascadeChunk) -> Option<Mesh> {
		Some(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))
	}
}

#[derive(Component, Clone)]
pub struct Wall<T: Material> {
	mesh: WallMesh,
	material: MeshMaterial3d<T>,
	wall_cache: HandleMap<WallMesh>,
}

impl<T: Material> Debug for Wall<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Wall<{}>", std::any::type_name::<T>())
	}
}

impl<T: Material> PartialEq for Wall<T> {
	fn eq(&self, other: &Self) -> bool {
		self.mesh == other.mesh && self.material == other.material
	}
}

impl<T: Material> Eq for Wall<T> {}

impl<T: Material> Hash for Wall<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.mesh.hash(state);
		self.material.hash(state);
	}
}

impl<T: Material> Wall<T> {
	pub fn new(material: MeshMaterial3d<T>) -> Self {
		Self { mesh: WallMesh::new(), material, wall_cache: HandleMap::new() }
	}

	pub fn with_wall_cache(mut self, wall_cache: HandleMap<WallMesh>) -> Self {
		self.wall_cache = wall_cache;
		self
	}
}

impl<T: Material> RenderItem for Wall<T> {
	fn spawn_render_items(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
	) -> Vec<Entity> {
		let mut render_items = Vec::new();

		let mesh_handle =
			MeshHandle::new(self.mesh.clone()).with_handle_cache(self.wall_cache.clone());

		render_items.push(
			commands
				.spawn((
					cascade_chunk.clone(),
					MeshDispatch::new(mesh_handle),
					transform,
					MeshMaterial3d(self.material.0.clone()),
				))
				.id(),
		);
		render_items
	}
}

impl<T: Material> Floor for Wall<T> {}

impl<T: Material> Partition for Wall<T> {}
