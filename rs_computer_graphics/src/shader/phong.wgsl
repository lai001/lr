struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texCoord: vec2<f32>,
    @location(1) vertexColor: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) fragPosition: vec3<f32>,
};

struct PhongShadingVSHConstants
{
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> phongShadingVshConstants: PhongShadingVSHConstants;

@vertex fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) texCoord: vec2<f32>,
    @location(2) vertexColor: vec4<f32>,
    @location(3) normal: vec3<f32>,
) -> VertexOutput {

    let mv = phongShadingVshConstants.view * phongShadingVshConstants.model;
    let mvp = phongShadingVshConstants.projection * mv;
    var result: VertexOutput;
    result.texCoord = texCoord;
    result.position = mvp * vec4<f32>(position, 1.0);
    result.vertexColor = vertexColor;
    result.normal = normal;
    result.fragPosition = (phongShadingVshConstants.model * vec4<f32>(position, 1.0)).xyz;
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
