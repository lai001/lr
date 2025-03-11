#include "common.wgsl"
#include "global_constants.wgsl"

struct VertexIn {
    @location(0) position: vec3<f32>,
#ifdef SKELETON_MAX_BONES
    @location(1) bone_ids: vec4<i32>,
    @location(2) bone_weights: vec4<f32>,
#endif    
};

struct Constants {
    model: mat4x4<f32>,
    id: u32,
};

#ifdef SKELETON_MAX_BONES
struct SkinConstants {
    bones: array<mat4x4<f32>, SKELETON_MAX_BONES>,
};
#endif

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>
};

@group(0) @binding(0) var<uniform> global_constants: GlobalConstants;

@group(0) @binding(1) var<uniform> constants: Constants;

#ifdef SKELETON_MAX_BONES
@group(0) @binding(2) var<uniform> skin_constants: SkinConstants;
#endif

@vertex fn vs_main(vertex_in: VertexIn) -> VertexOutput {
#ifdef SKELETON_MAX_BONES
    var bone_transform = skin_constants.bones[vertex_in.bone_ids[0]] * vertex_in.bone_weights[0];
    bone_transform += skin_constants.bones[vertex_in.bone_ids[1]] * vertex_in.bone_weights[1];
    bone_transform += skin_constants.bones[vertex_in.bone_ids[2]] * vertex_in.bone_weights[2];
    bone_transform += skin_constants.bones[vertex_in.bone_ids[3]] * vertex_in.bone_weights[3];
#endif
#ifdef PLAYER_VIEW
    let mvp = global_constants.view_projection * constants.model;
#else
    let mvp = global_constants.light_space_matrix * constants.model;
#endif
    var vertex_output: VertexOutput;
#ifdef SKELETON_MAX_BONES
    vertex_output.position = mvp * bone_transform * vec4<f32>(vertex_in.position, 1.0);
#else
    vertex_output.position = mvp * vec4<f32>(vertex_in.position, 1.0);
#endif
    return vertex_output;
}

@fragment fn fs_main() {

}