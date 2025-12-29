pub mod map;

use crate::mesh::IdentifiedMesh;
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;

pub trait MeshHandleCache: Clone + IdentifiedMesh {
	/// Caches a mesh handle.
	fn cache_mesh_handle(&self, mesh_handle: Handle<Mesh>, cascade_chunk: &CascadeChunk);

	/// Fetches a mesh handle from the cache.
	fn fetch_cached_mesh_handle(&self, cascade_chunk: &CascadeChunk) -> Option<Handle<Mesh>>;
}
