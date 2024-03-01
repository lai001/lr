const U32_MAX: u32 = 4294967295;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

struct VSConstants {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    physical_texture_size: u32,
    scene_factor: u32,
    feedback_bias: f32,
    id: u32,
};

fn mipmap_level(uv: vec2<f32>, texture_size: vec2<f32>) -> f32 {
    let s = dpdx(uv) * texture_size;
    let t = dpdy(uv) * texture_size;
    let delta = max(dot(s, s), dot(t, t));
    return 0.5 * log2(delta);
}

@group(0) @binding(0) var<uniform> constants: VSConstants;

@vertex 
fn vs_main(
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
    result.tex_coord = tex_coord;
    result.position = mvp * vec4<f32>(position, 1.0);
    return result;
}

@fragment 
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<u32> {
    let physical_texture_size = vec2<f32>(f32(constants.physical_texture_size / constants.scene_factor));
    let x: u32 = u32(f32(U32_MAX) * vertex.tex_coord.x);
    let y: u32 = u32(f32(U32_MAX) * vertex.tex_coord.y);
    let lod = mipmap_level(vertex.tex_coord, physical_texture_size);
    let color = vec4<u32>(u32(x), u32(y), u32(lod), constants.id);
    return color;
}
