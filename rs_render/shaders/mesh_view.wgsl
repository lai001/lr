#include "global_constants.wgsl"

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) @interpolate(flat) vertex_color0: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) vertex_color0: vec3<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>
};

struct Constants {
    model: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> global_constants: GlobalConstants;
@group(0) @binding(1) var<uniform> constants: Constants;

@vertex fn vs_main(vertex_in: VertexIn) -> VertexOutput {
    let mvp = global_constants.view_projection * constants.model;
    var result: VertexOutput;
    result.position = mvp * vec4<f32>(vertex_in.position, 1.0);
    result.vertex_color0 = vertex_in.vertex_color0;
    return result;
}

@fragment fn fs_main(vertex: VertexOutput) -> FragmentOutput {
    var fragment_output: FragmentOutput;
    fragment_output.color = vec4<f32>(vertex.vertex_color0, 1.0);
    return fragment_output;
}
