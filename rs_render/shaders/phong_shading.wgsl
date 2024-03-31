#include "common.wgsl"
#include "virtual_texture.wgsl"

struct VertexIn {
    @location(0) vertex_color: vec4<f32>,
    @location(1) position: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) tex_coord: vec2<f32>,
#ifdef SKELETON_MAX_BONES
    @location(6) bone_ids: vec4<i32>,
    @location(7) bone_weights: vec4<f32>,
#endif    
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) vertex_color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) frag_position: vec3<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>
};

struct Constants
{
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    physical_texture_size: vec2<f32>,
    diffuse_texture_size: vec2<f32>,
    diffuse_texture_max_lod: u32,
    is_virtual_diffuse_texture: u32,
    specular_texture_size: vec2<f32>,
    specular_texture_max_lod: u32,
    is_virtual_specular_texture: u32,
    tile_size: f32,
    is_enable_virtual_texture: i32,
#ifdef SKELETON_MAX_BONES
    bones: array<mat4x4<f32>, SKELETON_MAX_BONES>,
#endif
};

@group(0) @binding(0) var<uniform> constants: Constants;

@group(1) @binding(0) var diffuse_texture: texture_2d<f32>;

@group(1) @binding(1) var specular_texture: texture_2d<f32>;

@group(2) @binding(0) var physical_texture: texture_2d<f32>;

@group(2) @binding(1) var page_table_texture: texture_2d<u32>;

@group(3) @binding(0) var base_color_sampler: sampler;

@vertex fn vs_main(vertex_in: VertexIn) -> VertexOutput {
#ifdef SKELETON_MAX_BONES
    var bone_transform = constants.bones[vertex_in.bone_ids[0]] * vertex_in.bone_weights[0];
    bone_transform += constants.bones[vertex_in.bone_ids[1]] * vertex_in.bone_weights[1];
    bone_transform += constants.bones[vertex_in.bone_ids[2]] * vertex_in.bone_weights[2];
    bone_transform += constants.bones[vertex_in.bone_ids[3]] * vertex_in.bone_weights[3];
#endif

    let mv = constants.view * constants.model;
    let mvp = constants.projection * mv;
    var result: VertexOutput;
    result.tex_coord = vertex_in.tex_coord;
    result.vertex_color = vertex_in.vertex_color;
#ifdef SKELETON_MAX_BONES
    result.position = mvp * bone_transform * vec4<f32>(vertex_in.position, 1.0);
    result.frag_position = (constants.model * bone_transform * vec4<f32>(vertex_in.position, 1.0)).xyz;
    result.normal = (transpose(inverse(constants.model * bone_transform)) * vec4<f32>(vertex_in.normal, 0.0)).xyz;
#else
    result.position = mvp * vec4<f32>(vertex_in.position, 1.0);
    result.frag_position = (constants.model * vec4<f32>(vertex_in.position, 1.0)).xyz;
    result.normal = vertex_in.normal;
#endif 
    return result;
}

@fragment fn fs_main(vertex: VertexOutput) -> FragmentOutput {
    var fragment_output: FragmentOutput;
    if constants.is_enable_virtual_texture == 1 {
        if constants.is_virtual_diffuse_texture == 1 {
            let diffuse_color = virtual_texture_sample(vertex.tex_coord, constants.diffuse_texture_max_lod, constants.diffuse_texture_size);
            fragment_output.color = diffuse_color;
        } else {
            let diffuse_color = textureSample(diffuse_texture, base_color_sampler, vertex.tex_coord);
            fragment_output.color = diffuse_color;
        }
    } else {
        let diffuse_color = textureSample(diffuse_texture, base_color_sampler, vertex.tex_coord);
        let specular_color = textureSample(specular_texture, base_color_sampler, vertex.tex_coord);
        fragment_output.color = diffuse_color;
    }
    return fragment_output;
}
