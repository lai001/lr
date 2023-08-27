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
    mipmap_level_bias: f32,
    mipmap_level_scale: f32,
    feedback_bias: f32,
};

struct FeedBack {
    max_lod: f32,
    min_lod: f32,
    aniso_lod: f32,
    desired_lod: f32,
};

fn get_feed_back(uv: vec2<f32>, texture_size: vec2<f32>, feedback_bias: f32) -> FeedBack {
    var max_aniso = 4.0;
    var max_aniso_log2 = log2(max_aniso);
    var feed_back: FeedBack;
    var texcoords = uv * texture_size;
    var dx = dpdx(texcoords);
    var dy = dpdy(texcoords);
    var px = dot(dx, dx);
    var py = dot(dy, dy);
    feed_back.min_lod = 0.5 * log2(min(px, py));
    feed_back.max_lod = 0.5 * log2(max(px, py));
    feed_back.aniso_lod = feed_back.max_lod - min(feed_back.max_lod - feed_back.min_lod, max_aniso_log2);
    feed_back.desired_lod = max(feed_back.aniso_lod + feedback_bias, 0.0);
    return feed_back;
}

fn mipmap_level(uv: vec2<f32>, texture_size: vec2<f32>) -> f32 {
    var s = dpdx(uv) * texture_size;
    var t = dpdy(uv) * texture_size;
    var delta = max(dot(s, s), dot(t, t));
    return 0.5 * log2(delta);
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    var K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    var p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
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
    var page_x = vertex.tex_coord.x / f32(constants.tile_size);
    var page_y = vertex.tex_coord.y / f32(constants.tile_size);
    var virtual_texture_size = vec2<f32>(f32(constants.virtual_texture_size));
    // var mip = max(mipmap_level(vertex.tex_coord / virtual_texture_size, virtual_texture_size) * constants.mipmap_level_scale + constants.mipmap_level_bias, 0.0);
    var feed_back = get_feed_back(vertex.tex_coord / virtual_texture_size, virtual_texture_size, constants.feedback_bias);
    var color = vec4<u32>(u32(page_x), u32(page_y), u32(feed_back.desired_lod), u32(1));

    // var debug_color = hsv2rgb(vec3<f32>(mip / 8.0, 1.0, 1.0));
    // color.r = u32(debug_color.r * 65535.0);
    // color.g = u32(debug_color.g * 65535.0);
    // color.b = u32(debug_color.b * 65535.0);

    return color;
}
