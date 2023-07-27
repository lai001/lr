struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texCoord: vec3<f32>,
};

struct VSConstants
{
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
    var result: VertexOutput;
    result.position = (constants.projection * constants.view * vec4<f32>(position, 1.0)).xyww;
    result.texCoord = position;
    return result;
}

@group(1) @binding(0) var sky_box_texture_cube: texture_cube<f32>;

@group(2) @binding(0) var base_color_sampler: sampler;

@fragment fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(sky_box_texture_cube, base_color_sampler, vertex.texCoord);
    return color;
}
