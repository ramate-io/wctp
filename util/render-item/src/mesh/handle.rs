use crate::mesh::{
	cache::handle::map::HandleMap, IdentifiedMesh, MeshBuilder, MeshCache, MeshHandleCache, MeshId,
};
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;

#[derive(Debug, Clone, Component)]
pub struct MeshHandle<T: MeshBuilder + IdentifiedMesh + Clone> {
	handle_cache: HandleMap<T>,
	builder: T,
}

impl<T: MeshBuilder + IdentifiedMesh + Clone> MeshHandle<T> {
	pub fn new(builder: T) -> Self {
		Self { handle_cache: HandleMap::new(), builder }
	}

	pub fn with_handle_cache(mut self, handle_cache: HandleMap<T>) -> Self {
		self.handle_cache = handle_cache;
		self
	}
}

impl<T: MeshBuilder + IdentifiedMesh + Clone> IdentifiedMesh for MeshHandle<T> {
	fn id(&self) -> MeshId {
		self.builder.id()
	}
}

impl<T: MeshBuilder + IdentifiedMesh + Clone> MeshCache for MeshHandle<T> {
	fn cache_mesh(&self, _mesh: &Mesh, _cascade_chunk: &CascadeChunk) {
		// do nothing for now
	}

	fn fetch_cached_mesh(&self, _cascade_chunk: &CascadeChunk) -> Option<Mesh> {
		// do nothing for now
		None
	}
}

impl<T: MeshBuilder + IdentifiedMesh + Clone> MeshHandleCache for MeshHandle<T> {
	fn cache_mesh_handle(&self, mesh_handle: Handle<Mesh>, cascade_chunk: &CascadeChunk) {
		self.handle_cache.insert(cascade_chunk, &self.builder, mesh_handle);
	}

	fn fetch_cached_mesh_handle(&self, cascade_chunk: &CascadeChunk) -> Option<Handle<Mesh>> {
		self.handle_cache.get(cascade_chunk, &self.builder)
	}
}
