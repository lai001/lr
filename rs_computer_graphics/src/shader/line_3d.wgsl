struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) vertex_color: vec4<f32>,
};

struct Constants {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) 
@binding(0) 
var<uniform> constants: Constants;

@vertex 
fn vs_main(
    @location(0) vertex_color: vec4<f32>,
    @location(1) position: vec3<f32>,
) -> VertexOutput {
    let vp = constants.projection * constants.view;
    var result: VertexOutput;
    result.position = vp * vec4<f32>(position, 1.0);
    result.vertex_color = vertex_color;
    return result;
}

@fragment 
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.vertex_color;
}
