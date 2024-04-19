struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) vertex_color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) frag_position: vec3<f32>,
};

struct Constants
{
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>
};

@group(0) @binding(0) var<uniform> constants: Constants;

@vertex fn vs_main(vertex_in: VertexIn) -> VertexOutput {
    let mv = constants.view * constants.model;
    let mvp = constants.projection * mv;
    var result: VertexOutput;
    result.tex_coord = vertex_in.tex_coord;
    result.position = mvp * vec4<f32>(vertex_in.position, 1.0);
    result.frag_position = (constants.model * vec4<f32>(vertex_in.position, 1.0)).xyz;
    return result;
}

@fragment fn fs_main(vertex: VertexOutput) -> FragmentOutput {
    var fragment_output: FragmentOutput;
    return fragment_output;
}
