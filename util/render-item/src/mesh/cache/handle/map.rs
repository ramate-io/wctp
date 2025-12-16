use crate::mesh::{IdentifiedMesh, MeshId};
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkMeshKey<T: IdentifiedMesh + Clone> {
	chunk: CascadeChunk,
	mesh_id: MeshId,
	phantom: std::marker::PhantomData<T>,
}

impl<T: IdentifiedMesh + Clone> ChunkMeshKey<T> {
	pub fn new(chunk: CascadeChunk, mesh_id: MeshId) -> Self {
		Self { chunk, mesh_id, phantom: std::marker::PhantomData }
	}
}

#[derive(Debug, Clone)]
pub struct HandleMap<T: IdentifiedMesh + Clone> {
	cache: Arc<RwLock<HashMap<ChunkMeshKey<T>, Handle<Mesh>>>>,
}

impl<T: IdentifiedMesh + Clone> HandleMap<T> {
	pub fn new() -> Self {
		Self { cache: Arc::new(RwLock::new(HashMap::new())) }
	}
}

impl<T: IdentifiedMesh + Clone> HandleMap<T> {
	pub fn get(&self, chunk: &CascadeChunk, mesh_builder: &T) -> Option<Handle<Mesh>> {
		let cache = self.cache.read().unwrap();
		cache.get(&ChunkMeshKey::new(chunk.clone(), mesh_builder.id())).cloned()
	}

	pub fn insert(&self, chunk: &CascadeChunk, mesh_builder: &T, mesh: Handle<Mesh>) {
		let mut cache = self.cache.write().unwrap();
		cache.insert(ChunkMeshKey::new(chunk.clone(), mesh_builder.id()), mesh);
	}
}
