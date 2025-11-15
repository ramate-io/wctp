#import proc::marching_cubes 
#import proc::perlin_terrain

struct Sampling3D {
    chunk_origin : vec3<f32>,   // world offset
    chunk_size   : vec3<f32>,   // actual size in world units
    resolution   : vec3<u32>,   // voxel resolution (nx, ny, nz)
};

@group(0) @binding(0)
var<uniform> sampling : Sampling3D;

@group(0) @binding(1)
var<storage, read_write> cube_index : array<u32>;   // 1 u32 per voxel

@group(0) @binding(2)
var<storage, read_write> tri_counts : array<u32>;   // 1 u32 per voxel

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
    // --- Out-of-bounds guard ---
    if (gid.x >= sampling.resolution.x - 1u ||
        gid.y >= sampling.resolution.y - 1u ||
        gid.z >= sampling.resolution.z - 1u)
    {
        return;
    }

    // --- Convert voxel index → world coordinates ---
    let grid_f = vec3<f32>(f32(gid.x), f32(gid.y), f32(gid.z));
    let cell_size = sampling.chunk_size / vec3<f32>(sampling.resolution);

    // Evaluate SDF at the 8 cube corners
    let p = sampling.chunk_origin + grid_f * cell_size;

    // 8 samples
    let d000 = sdf(p + cell_size * vec3<f32>(0.0, 0.0, 0.0));
    let d100 = sdf(p + cell_size * vec3<f32>(1.0, 0.0, 0.0));
    let d010 = sdf(p + cell_size * vec3<f32>(0.0, 1.0, 0.0));
    let d110 = sdf(p + cell_size * vec3<f32>(1.0, 1.0, 0.0));
    let d001 = sdf(p + cell_size * vec3<f32>(0.0, 0.0, 1.0));
    let d101 = sdf(p + cell_size * vec3<f32>(1.0, 0.0, 1.0));
    let d011 = sdf(p + cell_size * vec3<f32>(0.0, 1.0, 1.0));
    let d111 = sdf(p + cell_size * vec3<f32>(1.0, 1.0, 1.0));

    // --- Cube Index: which corners are inside? ---
    var cube : u32 = 0u;
    if (d000 < 0.0) { cube |= 1u; }
    if (d100 < 0.0) { cube |= 2u; }
    if (d010 < 0.0) { cube |= 4u; }
    if (d110 < 0.0) { cube |= 8u; }
    if (d001 < 0.0) { cube |= 16u; }
    if (d101 < 0.0) { cube |= 32u; }
    if (d011 < 0.0) { cube |= 64u; }
    if (d111 < 0.0) { cube |= 128u; }

    // Compute flat index for storage
    let nx = sampling.resolution.x;
    let ny = sampling.resolution.y;
    let flat = gid.x + gid.y * nx + gid.z * nx * ny;

    // Write cube index
    cube_index[flat] = cube;

    // Look up triangle count using triTable
    let count = marching_cube_triangle_count(cube);
    tri_counts[flat] = count;
}
