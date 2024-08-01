#include "global_constants.wgsl"

struct VertexIn {
    @location(0) position: vec3<f32>,
	@location(1) vertex_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
	@location(0) vertex_color: vec4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

struct Constants {
    model: mat4x4<f32>,
    id: u32,
};

@group(0) @binding(0) var<uniform> global_constants: GlobalConstants;

@group(1) @binding(0) var<uniform> constants: Constants;

@vertex fn vs_main(vertex_in: VertexIn) -> VertexOutput {
    let mvp = global_constants.view_projection * constants.model;
    var vertex_output: VertexOutput;
    vertex_output.position = mvp * vec4<f32>(vertex_in.position, 1.0);
    vertex_output.vertex_color = vertex_in.vertex_color;
    return vertex_output;
}

@fragment fn fs_main(vertex_output: VertexOutput) -> FragmentOutput {
    var fragment_output: FragmentOutput;
    fragment_output.color = vertex_output.vertex_color;
    return fragment_output;
}