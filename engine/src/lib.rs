pub mod cascade;
pub mod chunk;
pub mod chunk_manager;
mod cpu;
mod gpu;
mod marching_cubes;
mod mesh_generator;
pub mod pipeline;
pub mod shaders;
mod terrain;

pub use chunk::{ChunkConfig, ChunkCoord, LoadedChunks};
pub use chunk_manager::{ChunkResolutionConfig, SdfResource, manage_chunks};
pub use sdf;

// Main exports for the engine
// Users should register:
// - ChunkConfig resource
// - ChunkResolutionConfig resource  
// - SdfResource<S> resource (where S: Sdf + Send + Sync)
// - LoadedChunks resource
// - Then add manage_chunks system to their Update schedule
