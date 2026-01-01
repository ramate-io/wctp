//---------------------------------------------------------
// Stylized Leaf Material Shader
// 
// This shader creates a stylized, abstract leaf appearance
// suitable for balls of intersecting planes. It uses UV-based
// alpha shaping to create leaf silhouettes from simple planes.
// Uses normal bending to fake volume on flat discs.
// Uses simple directional lighting with rim lighting.
//---------------------------------------------------------

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
    pbr_types::{PbrInput, pbr_input_new, STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT},
    pbr_functions as fns,
    pbr_bindings,
}
#import bevy_core_pipeline::tonemapping::tone_mapping


//---------------------------------------------------------
// Material Uniforms
//---------------------------------------------------------
@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> base_color: vec4<f32>;


//---------------------------------------------------------
// Perlin Noise
//---------------------------------------------------------
// Classic Perlin noise using gradient vectors at grid points
fn hash22(p: vec2<f32>) -> vec2<f32> {
    let p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    let dot_val = dot(p3, p3 + 33.33);
    let p3_xy = vec2<f32>(p3.x, p3.y);
    let p3_yz = vec2<f32>(p3.y, p3.z);
    return fract((p3_xy + p3_yz) * vec2<f32>(dot_val, dot_val * 1.618));
}

// Get gradient vector at grid point
fn grad(p: vec2<f32>) -> vec2<f32> {
    let h = hash22(p);
    // Map to gradient vectors - use angle from hash
    let angle = h.x * 6.28318; // 2 * PI
    return vec2<f32>(cos(angle), sin(angle));
}

fn perlin_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    var f = fract(p);
    
    // Smooth interpolation (smoothstep)
    f = f * f * (3.0 - 2.0 * f);
    
    // Get gradients at the four corners
    let g00 = grad(i);
    let g10 = grad(i + vec2<f32>(1.0, 0.0));
    let g01 = grad(i + vec2<f32>(0.0, 1.0));
    let g11 = grad(i + vec2<f32>(1.0, 1.0));
    
    // Distance vectors from grid points
    let d00 = f;
    let d10 = f - vec2<f32>(1.0, 0.0);
    let d01 = f - vec2<f32>(0.0, 1.0);
    let d11 = f - vec2<f32>(1.0, 1.0);
    
    // Dot products
    let n00 = dot(g00, d00);
    let n10 = dot(g10, d10);
    let n01 = dot(g01, d01);
    let n11 = dot(g11, d11);
    
    // Bilinear interpolation
    let nx0 = mix(n00, n10, f.x);
    let nx1 = mix(n01, n11, f.x);
    return mix(nx0, nx1, f.y);
}

// Fractal Perlin noise (multi-octave)
fn fractal_noise(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 10.0;
    
    // Multiple octaves for fractal detail
    for (var i = 0; i < 5; i++) {
        value += perlin_noise(p * frequency) * amplitude;
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    
    // Perlin noise is in range [-1, 1], normalize to [0, 1]
    return value * 0.5 + 0.5;
}


//---------------------------------------------------------
// Fragment Shader
//---------------------------------------------------------
@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    mesh: VertexOutput
) -> @location(0) vec4<f32> {

    //-----------------------------------------------------
    // 1. Calculate leaf shape alpha from noise
    //-----------------------------------------------------
    // Sample noise at UV coordinates
    let noise_scale = 6.0;
    let noise_value = fractal_noise(mesh.uv * noise_scale);
    
    // Threshold: above = visible, below = transparent
    let threshold = 0.50;
    
    // If noise is above threshold, visible, otherwise transparent
    let alpha = step(threshold, noise_value);
    
    // Early exit optimization: if fully transparent, skip lighting
    if (alpha < 0.001) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }


    //-----------------------------------------------------
    // 2. Normal bending (fake leaf volume)
    //-----------------------------------------------------
    // Compute fake spherical normal from center of disc outward
    // This makes flat discs appear to have volume through lighting
    
    // Get position relative to disc center (UV center is 0.5, 0.5)
    let uv_center = vec2<f32>(0.5, 0.5);
    let offset_from_center = (mesh.uv - uv_center) * 2.0; // Maps to [-1, 1] range
    let dist_from_center = length(offset_from_center);
    
    // Clamp distance to prevent issues at edges
    let clamped_dist = min(dist_from_center, 0.99);
    
    // Compute fake spherical normal in local space (disc plane is XY, normal is Z)
    // For a hemisphere, Z component is sqrt(1 - r^2) where r is distance from center
    let z_component = sqrt(1.0 - clamped_dist * clamped_dist);
    let fake_normal_local = normalize(vec3<f32>(
        offset_from_center.x,
        offset_from_center.y,
        z_component * select(-1.0, 1.0, is_front) // Flip based on front/back
    ));
    
    // Transform fake normal to world space
    // We need to construct a basis from the real normal
    let real_normal = normalize(mesh.world_normal);
    
    // Create tangent and bitangent vectors for the disc plane
    // Use a stable method: pick an arbitrary vector not parallel to normal
    var tangent_candidate = vec3<f32>(1.0, 0.0, 0.0);
    if (abs(real_normal.x) > 0.9) {
        tangent_candidate = vec3<f32>(0.0, 1.0, 0.0);
    }
    let tangent = normalize(tangent_candidate);
    
    // Build orthonormal basis: normal, tangent, bitangent
    let bitangent = normalize(cross(real_normal, tangent));
    let corrected_tangent = normalize(cross(bitangent, real_normal));
    
    // Transform fake normal from local space (XY plane) to world space
    // Local X -> tangent, Local Y -> bitangent, Local Z -> normal
    let fake_normal_world = normalize(
        fake_normal_local.x * corrected_tangent +
        fake_normal_local.y * bitangent +
        fake_normal_local.z * real_normal
    );
    
    // Blend between real normal and fake normal
    // Use distance from center to control blend - more bending at edges
    let bend_amount = 0.7; // Control strength of normal bending (0.0 = flat, 1.0 = fully spherical)
    let blend_factor = dist_from_center * bend_amount; // More bending at edges
    let blended_normal = normalize(mix(real_normal, fake_normal_world, blend_factor));


    //-----------------------------------------------------
    // 3. Build PBR input (same way StandardMaterial does)
    //-----------------------------------------------------
    var pbr_input: PbrInput = pbr_input_new();

    // Set base color
    pbr_input.material.base_color = base_color;

    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    // Basic PBR required fields
    pbr_input.frag_coord = mesh.position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.world_normal = fns::prepare_world_normal(
        blended_normal, // Use blended normal instead of real normal
        double_sided,
        is_front,
    );
    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.N = normalize(pbr_input.world_normal);
    pbr_input.V = fns::calculate_view(mesh.world_position, pbr_input.is_orthographic);


    //-----------------------------------------------------
    // 4. Compute PBR lighting (includes shadows)
    //-----------------------------------------------------
    let lit_color = fns::apply_pbr_lighting(pbr_input);


    //-----------------------------------------------------
    // 5. Apply alpha and output
    //-----------------------------------------------------
    let output_color = vec4<f32>(lit_color.rgb, base_color.a * alpha);

    // Apply tonemapping, color grading, exposure
    return tone_mapping(output_color, view.color_grading);
}

