#include "global_constants.wgsl"
#include "common.wgsl"

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord0: vec2<f32>,
};

struct InstanceIn {
    @location(2) position: vec3<f32>,
    @location(3) color: vec4<f32>,
    @builtin(instance_index) inst_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord0: vec2<f32>,
    @location(1) @interpolate(flat) inst_index: u32,
    @location(2) color: vec4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> global_constants: GlobalConstants;

@vertex fn vs_main(vertex_in: VertexIn, instance_in: InstanceIn) -> VertexOutput {
    let mvp = global_constants.view_projection * make_translation_matrix_from_vec3(instance_in.position) * strip_matrix_location_ant(global_constants.view);
    var output: VertexOutput;
    output.tex_coord0 = vertex_in.tex_coord0;
    output.position = mvp * vec4<f32>(vertex_in.position, 1.0);
    output.inst_index = instance_in.inst_index;
    output.color = instance_in.color;
    return output;
}

@fragment fn fs_main(vertex_output: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    output.color = vertex_output.color;
    var mask = smoothstep(0.2, 0.1, distance(vec2<f32>(0.5), vertex_output.tex_coord0));
    output.color.w = mask;
    return output;
}