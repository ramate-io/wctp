use bevy::prelude::*;
use chunk::cascade::CascadeChunk;

pub trait MeshBuilder: Clone {
	/// Builds a raw mesh.
	fn build_mesh(&self, cascade_chunk: &CascadeChunk) -> Option<Mesh>;
}

pub trait MeshFetcher: Clone {
	/// Builds mesh if it doesn't exist or fetches from the assets. Returns the handle to the mesh.
	fn fetch_mesh(
		&self,
		meshes: &mut ResMut<Assets<Mesh>>,
		cascade_chunk: &CascadeChunk,
	) -> Option<Handle<Mesh>>;
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
