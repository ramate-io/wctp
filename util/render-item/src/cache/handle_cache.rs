use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkMeshKey<T: Hash + Eq + Clone> {
	chunk: CascadeChunk,
	mesh_builder: T,
}

#[derive(Debug, Clone)]
pub struct HandleCache<T: Hash + Eq + Clone> {
	cache: Arc<RwLock<HashMap<ChunkMeshKey<T>, Handle<Mesh>>>>,
}

impl<T: Hash + Eq + Clone> HandleCache<T> {
	pub fn new() -> Self {
		Self { cache: Arc::new(RwLock::new(HashMap::new())) }
	}
}

impl<T: Hash + Eq + Clone> HandleCache<T> {
	pub fn get(&self, chunk: &CascadeChunk, mesh_builder: &T) -> Option<Handle<Mesh>> {
		let cache = self.cache.read().unwrap();
		cache
			.get(&ChunkMeshKey { chunk: chunk.clone(), mesh_builder: mesh_builder.clone() })
			.cloned()
	}

	pub fn insert(&self, chunk: &CascadeChunk, mesh_builder: &T, mesh: Handle<Mesh>) {
		let mut cache = self.cache.write().unwrap();
		cache.insert(
			ChunkMeshKey { chunk: chunk.clone(), mesh_builder: mesh_builder.clone() },
			mesh,
		);
	}
}
