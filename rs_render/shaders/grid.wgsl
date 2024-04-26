#include "global_constants.wgsl"

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_position: vec3<f32>,
};

struct Constants {
    model: mat4x4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>
};

@group(0) @binding(0) var<uniform> global_constants: GlobalConstants;

const N: f32 = 10.0;
const SCALE = 1000.0;

fn grid_texture_grad_box(p: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> f32 {
    var w: vec2<f32> = max(abs(ddx), abs(ddy)) + 0.01;
    var a: vec2<f32> = p + 0.5 * w;                        
    var b: vec2<f32> = p - 0.5 * w;           
    var i: vec2<f32> = (floor(a) + min(fract(a) * N, vec2<f32>(1.0)) - floor(b) - min(fract(b) * N, vec2<f32>(1.0))) / (N * w);
    return (1.0 - i.x) * (1.0 - i.y);
}

fn grid_texture(p: vec2<f32>) -> f32 {
    var i: vec2<f32> = step(fract(p), vec2(1.0 / N));
    return (1.0 - i.x) * (1.0 - i.y);
}

@vertex fn vs_main(vertex_in: VertexIn) -> VertexOutput {
    var constants: Constants;
    constants.model = mat4x4<f32>(
        vec4<f32>(SCALE, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, SCALE, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
    let mvp = global_constants.view_projection * constants.model;
    var result: VertexOutput;
    result.position = mvp * vec4<f32>(vertex_in.position, 1.0);
    result.frag_position = (constants.model * vec4<f32>(vertex_in.position, 1.0)).xyz;
    return result;
}

@fragment fn fs_main(vertex: VertexOutput) -> FragmentOutput {
    var fragment_output: FragmentOutput;
    var pos = vertex.frag_position;
    var uv = 1.0 * pos.xz;
    let mask = grid_texture_grad_box(uv, dpdx(uv), dpdy(uv));
    fragment_output.color = vec4<f32>(vec3<f32>(mask), 1.0 - mask);
    if (pos.z < (1.0 / N) && pos.z > -(1.0 / N)) {
        if (pos.x < 0.0 && pos.x > -N) {
            fragment_output.color = vec4<f32>(0.5, 0.0, 0.0, 1.0);
        } else if (pos.x > 0.0 && pos.x < N) {
            fragment_output.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    }
    if (pos.x < (1.0 / N) && pos.x > -(1.0 / N)) {
        if (pos.z < 0.0 && pos.z > -N) {
            fragment_output.color = vec4<f32>(0.0, 0.0, 0.5, 1.0);
        } else if (pos.z > 0.0 && pos.z < N) {
            fragment_output.color = vec4<f32>(0.0, 0.0, 1.0, 1.0);
        }
    }
    return fragment_output;
}
