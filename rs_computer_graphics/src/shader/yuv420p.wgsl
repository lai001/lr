struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = tex_coord;
    result.position = vec4<f32>(position.x, position.y, 0.0, 1.0);
    return result;
}


@group(0)
@binding(0)
var y_color: texture_2d<f32>;

@group(0)
@binding(1)
var u_color: texture_2d<f32>;

@group(0)
@binding(2)
var v_color: texture_2d<f32>;

@group(1)
@binding(0)
var baseColorSampler : sampler;

fn yuv2rgb(yuv: vec3<f32>) -> vec3<f32> {
    let y = yuv.x - 0.0625;
    let u = yuv.y - 0.5;
    let v = yuv.z - 0.5;
    let r = 1.164 * y + 1.793 * v;
    let g = 1.164 * y - 0.213 * u - 0.533 * v;
    let b = 1.164 * y + 2.112 * u;
    return vec3<f32>(r, g, b);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let y = textureSample(y_color, baseColorSampler, vertex.tex_coord).x;
    let u = textureSample(u_color, baseColorSampler, vertex.tex_coord).x;
    let v = textureSample(v_color, baseColorSampler, vertex.tex_coord).x;
    let rgb = yuv2rgb(vec3<f32>(y, u, v));
    return vec4<f32>(rgb, 1.0);
}
