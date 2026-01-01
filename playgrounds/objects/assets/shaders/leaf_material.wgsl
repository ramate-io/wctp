//---------------------------------------------------------
// Stylized Leaf Material Shader (Outline Bands, Stable)
//---------------------------------------------------------

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
    pbr_types::{PbrInput, pbr_input_new},
    pbr_functions as fns,
}
#import bevy_core_pipeline::tonemapping::tone_mapping


//---------------------------------------------------------
// Material Uniforms
//---------------------------------------------------------
@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> base_color: vec4<f32>;


//---------------------------------------------------------
// Fractal Noise (unchanged, stable)
//---------------------------------------------------------
fn hash22(p: vec2<f32>) -> vec2<f32> {
    let p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    let d = dot(p3, p3 + 33.33);
    return fract((p3.xy + p3.yz) * vec2<f32>(d, d * 1.618));
}

fn grad(p: vec2<f32>) -> vec2<f32> {
    let h = hash22(p);
    let a = h.x * 6.2831853;
    return vec2<f32>(cos(a), sin(a));
}

fn perlin(p: vec2<f32>) -> f32 {
    let i = floor(p);
    var f = fract(p);
    f = f * f * (3.0 - 2.0 * f);

    let g00 = grad(i);
    let g10 = grad(i + vec2<f32>(1.0, 0.0));
    let g01 = grad(i + vec2<f32>(0.0, 1.0));
    let g11 = grad(i + vec2<f32>(1.0, 1.0));

    let d00 = f;
    let d10 = f - vec2<f32>(1.0, 0.0);
    let d01 = f - vec2<f32>(0.0, 1.0);
    let d11 = f - vec2<f32>(1.0, 1.0);

    let nx0 = mix(dot(g00, d00), dot(g10, d10), f.x);
    let nx1 = mix(dot(g01, d01), dot(g11, d11), f.x);
    return mix(nx0, nx1, f.y);
}

fn fractal_noise(p: vec2<f32>) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var f = 6.0;
    for (var i = 0; i < 4; i++) {
        v += perlin(p * f) * a;
        f *= 2.0;
        a *= 0.5;
    }
    return v * 0.5 + 0.5;
}


//---------------------------------------------------------
// Fragment Shader
//---------------------------------------------------------
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {

    let uv = mesh.uv;

    //-----------------------------------------------------
    // Noise value
    //-----------------------------------------------------
    let n = fractal_noise(uv * 6.0);

    //-----------------------------------------------------
    // Two thresholds
    //-----------------------------------------------------
    let inner = 0.55; // solid leaf
    let outer = 0.5; // cutoff edge

    // Fully outside
    if (n < outer) {
        discard;
    }

    //-----------------------------------------------------
    // Outline band mask
    //-----------------------------------------------------
    let band = smoothstep(outer, inner, n);

    // band ≈ 0 at outer edge, ≈ 1 in leaf interior

    //-----------------------------------------------------
    // Color modulation
    //-----------------------------------------------------
    // Darken near edges, full color inside
    let edge_darkening = mix(0.65, 1.0, band);

    //-----------------------------------------------------
    // PBR setup (stable)
    //-----------------------------------------------------
    var pbr: PbrInput = pbr_input_new();
    pbr.material.base_color = vec4<f32>(
        base_color.rgb * edge_darkening,
        base_color.a
    );

    pbr.frag_coord = mesh.position;
    pbr.world_position = mesh.world_position;

    // IMPORTANT: do NOT override normals
    pbr.N = normalize(pbr.world_normal);

    pbr.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr.V = fns::calculate_view(mesh.world_position, pbr.is_orthographic);

    //-----------------------------------------------------
    // Lighting
    //-----------------------------------------------------
    let lit = fns::apply_pbr_lighting(pbr);

    //-----------------------------------------------------
    // Output
    //-----------------------------------------------------
    return tone_mapping(
        vec4<f32>(lit.rgb, base_color.a),
        view.color_grading
    );
}
