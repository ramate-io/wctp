//---------------------------------------------------------
// Stylized Leaf Material Shader (Stable Double-Sided)
//---------------------------------------------------------

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
    pbr_types::{PbrInput, pbr_input_new, STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT},
    pbr_functions as fns,
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
    let g10 = grad(i + vec2(1.0, 0.0));
    let g01 = grad(i + vec2(0.0, 1.0));
    let g11 = grad(i + vec2(1.0, 1.0));

    let d00 = f;
    let d10 = f - vec2(1.0, 0.0);
    let d01 = f - vec2(0.0, 1.0);
    let d11 = f - vec2(1.0, 1.0);

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
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {

    //-----------------------------------------------------
    // UV handling (mirror on back face)
    //-----------------------------------------------------
    var uv = mesh.uv;

    //-----------------------------------------------------
    // Leaf alpha from noise (soft threshold)
    //-----------------------------------------------------
    let noise = fractal_noise(uv * 6.0);

    let threshold = 0.5;
    let softness  = 0.08;
    let alpha = smoothstep(threshold - softness, threshold + softness, noise);

    if (alpha < 0.01) {
        discard;
    }

    //-----------------------------------------------------
    // Fake spherical normal (local leaf volume)
    //-----------------------------------------------------
    let centered_uv = uv * 2.0 - vec2(1.0);
    let r = length(centered_uv);
    let clamped = min(r, 0.999);

    let z = sqrt(1.0 - clamped * clamped);
    let fake_local = normalize(vec3<f32>(centered_uv, z));

    //-----------------------------------------------------
    // Build tangent basis from real normal
    //-----------------------------------------------------
    let N = normalize(mesh.world_normal);
    let up = select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(N.y) > 0.9);
    let T = normalize(cross(up, N));
    let B = cross(N, T);

    let fake_world = normalize(
        fake_local.x * T +
        fake_local.y * B +
        fake_local.z * N
    );

    let bend_strength = 0.7;
    let bend = smoothstep(0.0, 1.0, r) * bend_strength;
    let safe_bend = min(bend, 0.05);
    let final_normal = normalize(mix(N, fake_world, safe_bend));

    //-----------------------------------------------------
    // PBR setup
    //-----------------------------------------------------
    var pbr: PbrInput = pbr_input_new();

    pbr.material.base_color = base_color;
    pbr.frag_coord = mesh.position;
    pbr.world_position = mesh.world_position;

    /*pbr.world_normal = fns::prepare_world_normal(
        fake_world,
        false,
        true
    );*/

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
    let out_color = vec4<f32>(lit.rgb, base_color.a * alpha);
    return tone_mapping(out_color, view.color_grading);
}
