use crate::NormalizeChunk;
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use std::hash::Hash;

pub trait MeshBuilder: Clone + NormalizeChunk {
	/// The actual implementation which builds the mesh.
	fn build_mesh_impl(&self, cascade_chunk: &CascadeChunk) -> Option<Mesh>;

	/// Builds a mesh by normalizing the chunk and then building the mesh.
	fn build_mesh(&self, cascade_chunk: &CascadeChunk) -> Option<Mesh> {
		let normalized_chunk = self.normalize_chunk(cascade_chunk);
		self.build_mesh_impl(&normalized_chunk)
	}
}

pub trait MeshFetcher: Clone + Hash + Eq {
	/// Builds mesh if it doesn't exist or fetches from the assets. Returns the handle to the mesh.
	fn fetch_mesh(
		&self,
		meshes: &mut ResMut<Assets<Mesh>>,
		cascade_chunk: &CascadeChunk,
	) -> Option<Handle<Mesh>>;
}

pub trait MeshCache: Clone + Hash + Eq {
	/// Caches a mesh.
	fn cache_mesh(&self, mesh: &Mesh, cascade_chunk: &CascadeChunk);

	/// Fetches a mesh from the cache.
	fn fetch_cached_mesh(&self, cascade_chunk: &CascadeChunk) -> Option<Mesh>;
}

pub trait MeshHandleCache: Clone + Hash + Eq {
	/// Caches a mesh handle.
	fn cache_mesh_handle(&self, mesh_handle: Handle<Mesh>, cascade_chunk: &CascadeChunk);

	/// Fetches a mesh handle from the cache.
	fn fetch_cached_mesh_handle(
		&self,
		meshes: &mut ResMut<Assets<Mesh>>,
		cascade_chunk: &CascadeChunk,
	) -> Option<Handle<Mesh>>;
}

/// If it's already defined how the mesh is built, cached, and fetched, this trait can be used to fetch the mesh.
impl<T: MeshBuilder + MeshCache + MeshHandleCache> MeshFetcher for T {
	fn fetch_mesh(
		&self,
		meshes: &mut ResMut<Assets<Mesh>>,
		cascade_chunk: &CascadeChunk,
	) -> Option<Handle<Mesh>> {
		// Check if the mesh handle is already cached.
		if let Some(mesh) = self.fetch_cached_mesh_handle(meshes, cascade_chunk) {
			return Some(mesh);
		}

		// Check if the mesh is already cached (this will most often get hit when the mesh is on disk).
		let mesh_handle = if let Some(mesh) = self.fetch_cached_mesh(cascade_chunk) {
			Some(meshes.add(mesh))
		} else {
			self.build_mesh(cascade_chunk).map(|mesh| {
				self.cache_mesh(&mesh, cascade_chunk);
				meshes.add(mesh)
			})
		};

		mesh_handle.map(|handle| {
			self.cache_mesh_handle(handle.clone(), cascade_chunk);
			handle
		})
	}
}

#[derive(Component)]
pub struct MeshDispatch<T: MeshFetcher> {
	fetcher: T,
}

impl<T: MeshFetcher> MeshDispatch<T> {
	pub fn new(fetcher: T) -> Self {
		Self { fetcher }
	}
}

/// Fetches a mesh and spawns it.
pub fn fetch_meshes<T: MeshFetcher, M: Material>(
	commands: &mut Commands,
	mesh_dispatch: &MeshDispatch<T>,
	cascade_chunk: &CascadeChunk,
	transform: Transform,
	meshes: &mut ResMut<Assets<Mesh>>,
	material: MeshMaterial3d<M>,
) {
	let mesh = mesh_dispatch.fetcher.fetch_mesh(meshes, cascade_chunk);
	if let Some(mesh) = mesh {
		commands.spawn((Mesh3d(mesh), transform, material));
	}
}
