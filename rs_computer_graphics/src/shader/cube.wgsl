struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texCoord: vec2<f32>,
};

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) texCoord: vec2<f32>,
    @location(2) vertexColor: vec4<f32>,
    @location(3) normal: vec3<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.texCoord = texCoord;
    result.position = transform * vec4<f32>(position, 1.0);
    return result;
}

@group(0)
@binding(1)
var r_color: texture_2d<f32>;

@group(0)
@binding(2)
var baseColorSampler : sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(r_color, baseColorSampler, vertex.texCoord);
    return tex;
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}