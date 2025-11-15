#define_import_path proc:marching_cubes;

// ============================================================================
// Marching Cubes lookup tables + helpers (WGSL version)
// ============================================================================

// Edge → (corner_a, corner_b)
const EDGE_VERTEX_INDICES : array<vec2<u32>, 12> = array<vec2<u32>, 12>(
    vec2<u32>(0u, 1u),
    vec2<u32>(1u, 2u),
    vec2<u32>(2u, 3u),
    vec2<u32>(3u, 0u),
    vec2<u32>(4u, 5u),
    vec2<u32>(5u, 6u),
    vec2<u32>(6u, 7u),
    vec2<u32>(7u, 4u),
    vec2<u32>(0u, 4u),
    vec2<u32>(1u, 5u),
    vec2<u32>(2u, 6u),
    vec2<u32>(3u, 7u)
);

// Canonical cube corner positions
const CUBE_CORNERS : array<vec3<f32>, 8> = array<vec3<f32>, 8>(
    vec3<f32>(0.0, 0.0, 0.0), // 0
    vec3<f32>(1.0, 0.0, 0.0), // 1
    vec3<f32>(1.0, 0.0, 1.0), // 2
    vec3<f32>(0.0, 0.0, 1.0), // 3
    vec3<f32>(0.0, 1.0, 0.0), // 4
    vec3<f32>(1.0, 1.0, 0.0), // 5
    vec3<f32>(1.0, 1.0, 1.0), // 6
    vec3<f32>(0.0, 1.0, 1.0)  // 7
);

// ============================================================================
// Compute cube index (bitmask)
// corners[i] < 0 → inside
// ============================================================================
fn mc_get_cube_index(c0: f32, c1: f32, c2: f32, c3: f32,
                     c4: f32, c5: f32, c6: f32, c7: f32) -> u32 {

    var idx : u32 = 0u;

    if (c0 < 0.0) { idx = idx | 1u; }
    if (c1 < 0.0) { idx = idx | 2u; }
    if (c2 < 0.0) { idx = idx | 4u; }
    if (c3 < 0.0) { idx = idx | 8u; }
    if (c4 < 0.0) { idx = idx | 16u; }
    if (c5 < 0.0) { idx = idx | 32u; }
    if (c6 < 0.0) { idx = idx | 64u; }
    if (c7 < 0.0) { idx = idx | 128u; }

    return idx;
}

// ============================================================================
// Linear interpolation along an edge
// edge        = edge index 0..11
// cube_origin = world-space origin of this cube
// cube_size   = size of cube in world units
// corner_vals = SDF values per corner
// ============================================================================
fn mc_interpolate_vertex(
    edge        : u32,
    cube_origin : vec3<f32>,
    cube_size   : f32,
    corner_vals : array<f32, 8>
) -> vec3<f32> {

    let idx = EDGE_VERTEX_INDICES[edge];
    let a = idx.x;
    let b = idx.y;

    let v1 = CUBE_CORNERS[a];
    let v2 = CUBE_CORNERS[b];

    let val1 = corner_vals[a];
    let val2 = corner_vals[b];

    // If close to equal, use midpoint
    if (abs(val1 - val2) < 1e-6) {
        return cube_origin + (v1 + v2) * 0.5 * cube_size;
    }

    // Linear interpolation t = -v1 / (v2 - v1)
    let t_raw = (-val1) / (val2 - val1);
    let t     = clamp(t_raw, 0.0, 1.0);

    let local = v1 + (v2 - v1) * t;
    return cube_origin + local * cube_size;
}

// ============================================================================
// TRIANGULATION TABLE
// 256 entries, each a list of edge indices making triangles (terminated by -1)
// We use 15 columns just like your Rust version.
// ============================================================================
const TRIANGULATIONS : array<array<i32, 15>, 256> = array<array<i32, 15>, 256>(
    // --- ENTRY 0 ---
    array<i32, 15>(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
    // --- ENTRY 1 ---
    array<i32, 15>(0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
    // --- ENTRY 2 ---
    array<i32, 15>(0, 1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
    // ...
    // NOTE: Copy all 256 entries here exactly as in your Rust `TRIANGULATIONS`.
    // WGSL syntax matches Rust arrays closely.
    // ...
);
// (For brevity I'm not repeated all 256 here, but in your real file you paste all of them.)

// ============================================================================
// Count how many triangles for a cube index
// ============================================================================
fn mc_triangle_count(cube : u32) -> u32 {
    var c : u32 = 0u;
    let row = TRIANGULATIONS[cube];

    // Each 3 entries form a triangle. Stop at -1 sentinel.
    for (var i : u32 = 0u; i < 15u; i = i + 3u) {
        if (row[i] == -1) {
            break;
        }
        c = c + 1u;
    }
    return c;
}

// ============================================================================
// Look up the i'th triangle for a cube index
// Returns the 3 edge indices for that triangle
// ============================================================================
fn mc_get_triangle_edges(cube : u32, tri : u32) -> vec3<i32> {
    let base = tri * 3u;
    let row = TRIANGULATIONS[cube];
    return vec3<i32>(row[base], row[base + 1u], row[base + 2u]);
}
