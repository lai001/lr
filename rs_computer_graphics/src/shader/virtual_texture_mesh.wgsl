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
    mipmap_level_bias: f32,
    mipmap_level_scale: f32,
};

struct PhysicalPixelInfo {
    array_index: u32,
    tex_coord: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> constants: Constants;

@group(1) @binding(0) var page_table_texture: texture_2d_array<u32>;

@group(1) @binding(1) var physical_texture: texture_2d_array<f32>;

@group(2) @binding(0) var filterable_sampler: sampler;

fn mipmap_level(uv: vec2<f32>, texture_size: vec2<f32>) -> f32 {
    var s = dpdx(uv) * texture_size;
    var t = dpdy(uv) * texture_size;
    var delta = max(dot(s, s), dot(t, t));
    return 0.5 * log2(delta);
}

fn page_size(level: u32) -> u32 {
    return max(u32(1), constants.tile_size >> level);
}

fn remap_value_range(
    value: f32,
    from_range_lower: f32,
    from_range_upper: f32,
    to_range_lower: f32,
    to_range_upper: f32,
) -> f32 {
    return (value - from_range_lower) / (from_range_upper - from_range_lower)
        * (to_range_upper - to_range_lower)
        + to_range_lower;
}

fn get_physical_pixel_info(virtual_tex_coord: vec2<f32>, virtual_page: vec2<u32>, level: i32) -> PhysicalPixelInfo {
    var indirect = textureLoad(page_table_texture, vec2<i32>(virtual_page), level, 0);

    var sub_tile_size: u32 = page_size(u32(level));

    var physical_tex_coord = vec2<f32>(0.0);

    physical_tex_coord.x = f32(indirect.x) / f32(constants.physical_texture_size);
    physical_tex_coord.y = f32(indirect.y) / f32(constants.physical_texture_size);

    var offset_x = remap_value_range(virtual_tex_coord.x - f32(virtual_page.x * constants.tile_size), 0.0, f32(constants.tile_size), 0.0, f32(sub_tile_size) / f32(constants.physical_texture_size));
    var offset_y = remap_value_range(virtual_tex_coord.y - f32(virtual_page.y * constants.tile_size), 0.0, f32(constants.tile_size), 0.0, f32(sub_tile_size) / f32(constants.physical_texture_size));

    physical_tex_coord.x += offset_x;
    physical_tex_coord.y += offset_y;

    var pixel_info: PhysicalPixelInfo;
    pixel_info.array_index = indirect.z;
    pixel_info.tex_coord = physical_tex_coord;
    // pixel_info.color = textureSampleGrad(physical_texture, filterable_sampler, pixel_info.tex_coord, i32(pixel_info.array_index), vec2<f32>(5.0), vec2<f32>(5.0));
    pixel_info.color = textureSample(physical_texture, filterable_sampler, pixel_info.tex_coord, i32(pixel_info.array_index));
    // if (indirect.w != u32(1)) {
    //     pixel_info.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    // }
    // if (pixel_info.color.r == 0.0) {
    //     pixel_info.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    // }
    return pixel_info;
}

fn physical_texture_mipmap_sample(virtual_tex_coord: vec2<f32>, level: f32) -> vec4<f32> {
    var virtual_page = vec2<u32>(virtual_tex_coord / f32(constants.tile_size));
    var p: i32 = max(i32(level) - 1, 0);
    var m: i32 = i32(level);
    var n: i32 = i32(level) + 1;
    var pixel_info_p = get_physical_pixel_info(virtual_tex_coord, virtual_page, p);
    var pixel_info_m = get_physical_pixel_info(virtual_tex_coord, virtual_page, m);
    var pixel_info_n = get_physical_pixel_info(virtual_tex_coord, virtual_page,n);
    var color_0 = mix(pixel_info_m.color, pixel_info_n.color, fract(level));
    var color_1 = mix(pixel_info_p.color, pixel_info_m.color, fract(level));
    return (color_0 + color_1) / 2.0;
}

fn physical_texture_sample(virtual_tex_coord: vec2<f32>, level: f32) -> vec4<f32> {
    var virtual_page = vec2<u32>(floor(virtual_tex_coord / f32(constants.tile_size)));
    var pixel_info = get_physical_pixel_info(virtual_tex_coord, virtual_page, i32(level));
    return pixel_info.color;
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    var K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    var p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}

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

@fragment 
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var virtual_texture_size = vec2<f32>(f32(constants.virtual_texture_size));
    var bias = constants.mipmap_level_bias;
    var mip = max(mipmap_level(vertex.tex_coord / virtual_texture_size, virtual_texture_size) * constants.mipmap_level_scale + bias, 0.0);
    var color = physical_texture_mipmap_sample(vertex.tex_coord, mip);
    // var debug_color = hsv2rgb(vec3<f32>(mip / 8.0, 1.0, 1.0));
    // color.r = debug_color.r;
    // color.g = debug_color.g;
    // color.b = debug_color.b;
    return color;
}
