struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

struct VSConstants {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    physical_texture_size: u32,
    virtual_texture_size: u32,
    tile_size: u32,
    feed_back_texture_width: u32,
    feed_back_texture_height: u32,
};

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
    var page_x = vertex.tex_coord.x / f32(constants.tile_size);
    var page_y = vertex.tex_coord.y / f32(constants.tile_size);
    return vec4<u32>(u32(page_x), u32(page_y), u32(0), u32(1));
}
