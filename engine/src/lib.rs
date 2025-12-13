pub mod cascade;
pub mod chunk;
pub mod chunk_manager;
pub mod cpu;
pub mod marching_cubes;
pub mod shaders;

pub use chunk::{ChunkConfig, ChunkCoord, LoadedChunks};
pub use chunk_manager::{manage_chunks, ChunkResolutionConfig, SdfResource};
pub use sdf;

// Main exports for the engine
// Users should register:
// - ChunkConfig resource
// - ChunkResolutionConfig resource
// - SdfResource<S> resource (where S: Sdf + Send + Sync)
// - LoadedChunks resource
// - Then add manage_chunks system to their Update schedule
