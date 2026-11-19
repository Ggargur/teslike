// PS1-style extension for StandardMaterial: vertex snapping, optional affine UVs,
// colour quantisation and ordered dithering. Fog comes from the camera's DistanceFog.

import bevy_pbr::render::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    forward_io::{Vertex, UncompressedVertex, VertexOutput, FragmentOutput, decompress_vertex},
    view_transformations::position_world_to_clip,
    pbr_types,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_fragment::pbr_input_from_standard_material,
};

struct PsxParams {
    // xy: snap grid resolution in pixels (0 = off), z: colour levels per channel, w: dither strength.
    a: vec4<f32>,
    // x: affine texture mapping (0 or 1).
    b: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> psx: PsxParams;

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var vertex = decompress_vertex(vertex_no_morph, vertex_no_morph.instance_index);

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

@if(SKINNED)
    var world_from_local = skinning::skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex_no_morph.instance_index
    );
@else
    var world_from_local = mesh_world_from_local;

@if(VERTEX_NORMALS && SKINNED)
    out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
@if(VERTEX_NORMALS && !SKINNED)
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex_no_morph.instance_index
    );

@if(VERTEX_POSITIONS)
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
@if(VERTEX_POSITIONS)
    out.position = position_world_to_clip(out.world_position.xyz);

    // Snap to the screen-space grid (clip -> ndc -> grid -> clip). Skip vertices behind the camera.
    let grid = psx.a.xy * 0.5;
    if grid.x > 0.0 && grid.y > 0.0 && out.position.w > 0.0001 {
        let ndc = out.position.xy / out.position.w;
        out.position = vec4<f32>(round(ndc * grid) / grid * out.position.w, out.position.zw);
    }

@if(VERTEX_UVS_A)
    out.uv = vertex.uv;
    // Affine mapping: pre-multiply by w, undone in the fragment shader (see below).
@if(VERTEX_UVS_A)
    if psx.b.x > 0.5 {
        out.uv = vertex.uv * out.position.w;
        out.world_position.w = out.position.w;
    }
@if(VERTEX_UVS_B)
    out.uv_b = vertex.uv_b;

@if(VERTEX_TANGENTS)
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        vertex_no_morph.instance_index
    );

@if(VERTEX_COLORS)
    out.color = vertex.color;

@if(VERTEX_OUTPUT_INSTANCE_INDEX)
    out.instance_index = vertex_no_morph.instance_index;

@if(VISIBILITY_RANGE_DITHER)
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex_no_morph.instance_index, mesh_world_from_local[3]);

    return out;
}

fn bayer4(p: vec2<u32>) -> f32 {
    let x = p.x & 3u;
    let y = p.y & 3u;
    var m = array<f32, 16>(
         0.0,  8.0,  2.0, 10.0,
        12.0,  4.0, 14.0,  6.0,
         3.0, 11.0,  1.0,  9.0,
        15.0,  7.0, 13.0,  5.0,
    );
    return (m[y * 4u + x] + 0.5) / 16.0;
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn from_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fragment(
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var in = vertex_output;

@if(VERTEX_UVS_A)
    if psx.b.x > 0.5 {
        in.uv = in.uv / in.world_position.w;
    }
    in.world_position.w = 1.0;

    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );

    var out: FragmentOutput;
    if (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // Quantise in sRGB space, with a Bayer 4x4 ordered dither.
    let levels = max(psx.a.z, 2.0);
    if psx.a.z > 0.0 {
        let steps = levels - 1.0;
        let thr = 0.5 + (bayer4(vec2<u32>(in.position.xy)) - 0.5) * psx.a.w;
        let srgb = clamp(to_srgb(out.color.rgb), vec3<f32>(0.0), vec3<f32>(1.0));
        let q = floor(srgb * steps + thr) / steps;
        out.color = vec4<f32>(from_srgb(q), out.color.a);
    }
    return out;
}
