#include "global_constants.wgsl"

const U32_MAX: u32 = 4294967295;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
#ifdef SKELETON_MAX_BONES
    @location(2) bone_ids: vec4<i32>,
    @location(3) bone_weights: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

struct Constants
{
    model: mat4x4<f32>,
    diffuse_texture_size: vec2<f32>,
    diffuse_texture_max_lod: u32,
    is_virtual_diffuse_texture: u32,
    specular_texture_size: vec2<f32>,
    specular_texture_max_lod: u32,
    is_virtual_specular_texture: u32,
    id: u32,
#ifdef SKELETON_MAX_BONES
    bones: array<mat4x4<f32>, SKELETON_MAX_BONES>,
#endif
};

fn mipmap_level(uv: vec2<f32>, texture_size: vec2<f32>) -> f32 {
    let s = dpdx(uv) * texture_size;
    let t = dpdy(uv) * texture_size;
    let delta = max(dot(s, s), dot(t, t));
    return 0.5 * log2(delta);
}

@group(0) @binding(0) var<uniform> global_constants: GlobalConstants;

@group(1) @binding(0) var<uniform> constants: Constants;

@vertex
fn vs_main(vertex_in: VertexIn) -> VertexOutput {
#ifdef SKELETON_MAX_BONES
    var bone_transform = constants.bones[vertex_in.bone_ids[0]] * vertex_in.bone_weights[0];
    bone_transform += constants.bones[vertex_in.bone_ids[1]] * vertex_in.bone_weights[1];
    bone_transform += constants.bones[vertex_in.bone_ids[2]] * vertex_in.bone_weights[2];
    bone_transform += constants.bones[vertex_in.bone_ids[3]] * vertex_in.bone_weights[3];
#endif

    let mvp = global_constants.view_projection * constants.model;
    var result: VertexOutput;
    result.tex_coord = vertex_in.tex_coord;
    result.position = mvp * vec4<f32>(vertex_in.position, 1.0);
#ifdef SKELETON_MAX_BONES
    result.position = mvp * bone_transform * vec4<f32>(vertex_in.position, 1.0);
#else
    result.position = mvp * vec4<f32>(vertex_in.position, 1.0);
#endif
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<u32> {
    let physical_texture_size = vec2<f32>(global_constants.physical_texture_size / global_constants.scene_factor);
    let x: u32 = u32(f32(U32_MAX) * vertex.tex_coord.x);
    let y: u32 = u32(f32(U32_MAX) * vertex.tex_coord.y);
    let lod = mipmap_level(vertex.tex_coord, physical_texture_size);
    let color = vec4<u32>(u32(x), u32(y), u32(lod), constants.id);
    return color;
}
