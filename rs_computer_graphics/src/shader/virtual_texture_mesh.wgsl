struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) vertex_color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) frag_position: vec3<f32>,
};

struct Constants {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    physical_texture_size: u32,
    virtual_texture_size: u32,
    tile_size: u32,
};

@group(0) @binding(0) var<uniform> constants: Constants;

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
    result.vertex_color = vertex_color;
    result.normal = normal;
    result.frag_position = (constants.model * vec4<f32>(position, 1.0)).xyz;
    return result;
}

@group(1) @binding(0) var page_table_texture: texture_2d<u32>;

@group(1) @binding(1) var physical_texture: texture_2d<f32>;

@group(2) @binding(0) var filterable_sampler: sampler;

@fragment 
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var page_x = u32(vertex.tex_coord.x / f32(constants.tile_size));
    var page_y = u32(vertex.tex_coord.y / f32(constants.tile_size));
    var indirect = textureLoad(page_table_texture, vec2<i32>(i32(page_x), i32(page_y)), 0);

    var tex_coord = vec2<f32>(0.0);

    tex_coord.x = f32(indirect.x) * f32(constants.tile_size) / f32(constants.physical_texture_size);
    tex_coord.y = f32(indirect.y) * f32(constants.tile_size) / f32(constants.physical_texture_size);

    var offset_x = (vertex.tex_coord.x - f32(page_x * constants.tile_size)) / f32(constants.physical_texture_size);
    var offset_y = (vertex.tex_coord.y - f32(page_y * constants.tile_size)) / f32(constants.physical_texture_size);

    tex_coord.x += offset_x;
    tex_coord.y += offset_y;

    var color = textureSample(physical_texture, filterable_sampler, tex_coord).xyz;

    return vec4<f32>(color, 1.0);
}
