//---------------------------------------------------------
// Stylized Leaf Material Shader
// 
// This shader creates a stylized, abstract leaf appearance
// suitable for balls of intersecting planes. It uses UV-based
// alpha shaping to create leaf silhouettes from simple planes.
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
// Leaf Shape Function
//---------------------------------------------------------
// Creates a leaf shape using UV coordinates (0-1 range)
// Returns alpha value: 1.0 = inside leaf, 0.0 = outside leaf
// Uses a teardrop/ellipse shape that can be parameterized
fn leaf_shape(uv: vec2<f32>) -> f32 {
    // Center the UV coordinates around (0.5, 0.5)
    let centered = (uv - 0.5) * 2.0; // Now in range [-1, 1]
    
    // Create a leaf shape using distance from center
    // X-axis: wider at top, narrower at bottom (teardrop)
    // Y-axis: standard ellipse
    let x = centered.x;
    let y = centered.y;
    
    // Create a teardrop shape:
    // - Top half: wider ellipse
    // - Bottom half: narrower, more pointed
    
    // Distance from center along Y axis (0 = top, 1 = bottom)
    let y_progress = (y + 1.0) * 0.5; // Map [-1, 1] to [0, 1]
    
    // Width varies from top to bottom
    // Top (y_progress = 0): width = 1.0
    // Bottom (y_progress = 1): width = 0.3 (pointed)
    let width_factor = mix(1.0, 0.3, y_progress * y_progress); // Quadratic falloff for smooth taper
    
    // Create ellipse with varying width
    let ellipse = (x * x) / (width_factor * width_factor) + (y * y);
    
    // Create alpha mask with soft edges for anti-aliasing
    // Values < 0.8 = fully opaque, > 1.0 = fully transparent
    let alpha = 1.0 - smoothstep(0.8, 1.0, ellipse);
    
    // Add a subtle stem at the bottom
    // This creates a small stem-like extension at the base
    let stem_y = y + 0.9; // Position stem near bottom
    let stem_width = 0.15;
    let stem_alpha = 1.0 - smoothstep(0.0, stem_width, abs(x));
    let stem_mask = step(0.0, stem_y) * (1.0 - step(0.2, stem_y)) * stem_alpha;
    
    // Combine leaf and stem
    return max(alpha, stem_mask);
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
    // 1. Calculate leaf shape alpha from UV coordinates
    //-----------------------------------------------------
    // Get UV coordinates (should be in [0, 1] range for the plane)
    let uv = mesh.uv;
    
    // Calculate the leaf shape alpha
    // This determines which parts of the plane are visible
    let shape_alpha = leaf_shape(uv);
    
    // Early discard for fully transparent fragments (optimization)
    // Note: In WGSL, we can't use discard, but we can set alpha to 0
    // The GPU will optimize this away during blending
    
    
    //-----------------------------------------------------
    // 2. Prepare world-space normal (handles double-sided)
    //-----------------------------------------------------
    // Build PBR input to get proper normal handling
    var pbr_input: PbrInput = pbr_input_new();
    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;
    
    // Get world-space normal, flipping if needed for back faces
    let world_normal = fns::prepare_world_normal(
        mesh.world_normal,
        double_sided,
        is_front,
    );
    let N = normalize(world_normal);
    
    // Calculate view direction (from fragment to camera)
    let is_orthographic = view.clip_from_view[3].w == 1.0;
    let V = fns::calculate_view(mesh.world_position, is_orthographic);


    //-----------------------------------------------------
    // 3. Simple directional lighting
    //-----------------------------------------------------
    // Use a fixed light direction (pointing down and slightly forward)
    // This gives a consistent lighting feel across all leaves
    let light_dir = normalize(vec3<f32>(0.3, -0.8, 0.5));
    
    // Calculate how much the surface faces the light
    // Clamp to avoid negative values (no backlighting in this simple model)
    let NdotL = max(dot(N, light_dir), 0.0);
    
    // Create a stylized lighting curve
    // Using a smooth curve that gives soft, pleasant shading
    // The 0.3 ambient + 0.7 lit gives a nice balance
    let lighting = 0.3 + 0.7 * smoothstep(0.0, 0.5, NdotL);


    //-----------------------------------------------------
    // 4. Rim lighting (fresnel effect)
    //-----------------------------------------------------
    // Rim lighting makes edges glow, which is great for stylized leaves
    // It's strongest when viewing the surface edge-on
    let fresnel = 1.0 - abs(dot(N, V));
    
    // Apply a power curve to control rim intensity
    // Higher power = sharper rim, lower power = softer rim
    let rim_power = 2.5;
    let rim = pow(fresnel, rim_power);
    
    // Add rim lighting to the base color
    // Using a warm, slightly brighter tint for the rim
    let rim_color = vec3<f32>(1.1, 1.15, 1.0); // Slight warm/yellow tint
    let rim_contribution = rim * 0.3; // Control rim intensity (0.0 to 1.0)


    //-----------------------------------------------------
    // 5. Combine lighting and color
    //-----------------------------------------------------
    // Start with the base color
    var final_color = base_color.rgb;
    
    // Apply directional lighting
    final_color *= lighting;
    
    // Add rim lighting (additive, makes edges brighter)
    final_color = mix(final_color, final_color * rim_color, rim_contribution);
    
    // Optional: Add subtle color variation based on position
    // This gives a bit of organic variation without being too noisy
    // Using world position to create a subtle gradient-like effect
    let position_noise = sin(mesh.world_position.x * 2.0 + mesh.world_position.y * 3.0) * 0.05;
    final_color += position_noise * base_color.rgb;


    //-----------------------------------------------------
    // 6. Apply leaf shape alpha and output
    //-----------------------------------------------------
    // Combine the shape alpha with the base color alpha
    // This creates the leaf silhouette
    let final_alpha = base_color.a * shape_alpha;
    
    // Use Bevy's tonemapping to ensure proper color space conversion
    let output = tone_mapping(vec4<f32>(final_color, final_alpha), view.color_grading);
    
    return output;
}

