struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texCoord: vec2<f32>,
    @location(1) vertexColor: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) fragPosition: vec3<f32>,
};

struct VSConstants
{
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    bans: f32,
};

@group(0) @binding(0) var<uniform> constants: VSConstants;

@group(1) @binding(0) var audio_texture: texture_2d<f32>;

@group(2) @binding(0) var base_color_sampler : sampler;

fn visualization(uv: vec2<f32>, bans: f32) -> vec4<f32> {
    var color = mix(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 165.0 / 255.0, 0.0), uv.y);
    var frequency = textureSample(audio_texture, base_color_sampler, vec2<f32>(floor(uv.x * bans) / bans, 0.0)).x;
    if(uv.y < frequency) {
        return vec4<f32>(vec3<f32>(0.0), 1.0);
    } else {
        var mask = step(0.2, fract(uv.x * bans));
        var r = 1.0 + smoothstep(0.0, frequency, uv.y);
        return  vec4<f32>(r * color * mask, 1.0);
    }
}

@vertex fn vs_main(
    @location(0) vertex_color: vec4<f32>,
    @location(1) position: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) tex_coord: vec2<f32>,
) -> VertexOutput {
    let mv = constants.view * constants.model;
    let mvp = constants.projection * mv;
    var result: VertexOutput;
    result.texCoord = tex_coord;
    result.position = mvp * vec4<f32>(position, 1.0);
    result.vertexColor = vertex_color;
    result.normal = normal;
    result.fragPosition = (constants.model * vec4<f32>(position, 1.0)).xyz;
    return result;
}

@fragment fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = visualization(vertex.texCoord, constants.bans);
    return color;
}
