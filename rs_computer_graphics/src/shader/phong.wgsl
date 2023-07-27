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
};

@group(0) @binding(0) var<uniform> constants: VSConstants;

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

@group(1) @binding(0) var diffuseTexture: texture_2d<f32>;

@group(1) @binding(1) var specularTexture: texture_2d<f32>;

@group(2) @binding(0) var baseColorSampler : sampler;

@fragment fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let diffuseColor = textureSample(diffuseTexture, baseColorSampler, vertex.texCoord).xyz;
    let specularColor = textureSample(specularTexture, baseColorSampler, vertex.texCoord).xyz;
    return vec4<f32>(diffuseColor, 1.0);
}
