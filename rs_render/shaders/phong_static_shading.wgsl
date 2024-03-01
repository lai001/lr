struct VertexIn {
    @location(0) vertex_color: vec4<f32>,
    @location(1) position: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) tex_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) vertex_color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) frag_position: vec3<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>
};

struct Constants
{
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    physical_texture_size: vec2<f32>,
    diffuse_texture_size: vec2<f32>,
    diffuse_texture_max_lod: u32,
    is_virtual_diffuse_texture: u32,
    specular_texture_size: vec2<f32>,
    specular_texture_max_lod: u32,
    is_virtual_specular_texture: u32,
    tile_size: f32,
    is_enable_virtual_texture: i32,
};

@group(0) @binding(0) var<uniform> constants: Constants;

@group(1) @binding(0) var diffuse_texture: texture_2d<f32>;

@group(1) @binding(1) var specular_texture: texture_2d<f32>;

@group(2) @binding(0) var physical_texture: texture_2d<f32>;

@group(2) @binding(1) var page_table_texture: texture_2d<u32>;

@group(3) @binding(0) var base_color_sampler: sampler;

fn mipmap_level(uv: vec2<f32>, texture_size: vec2<f32>) -> f32 {
    let s = dpdx(uv) * texture_size;
    let t = dpdy(uv) * texture_size;
    let delta = max(dot(s, s), dot(t, t));
    return 0.5 * log2(delta);
}

fn get_mip_level_size(length: u32, level: u32) -> u32 {
    return max(u32(1), length >> level);
}

fn mipmap_size(lod0_size: vec2<f32>, level: u32) -> vec2<f32> {
    let x = get_mip_level_size(u32(lod0_size.x), level);
    let y = get_mip_level_size(u32(lod0_size.y), level);
    return vec2<f32>(f32(x), f32(y));
}

fn virtual_texture_sample(tex_coord: vec2<f32>, max_lod: u32, texture_size: vec2<f32>) -> vec4<f32> {
    let physical_size = constants.physical_texture_size;
    let lod = min(u32(mipmap_level(tex_coord, physical_size)), max_lod);
    let texture_mip_size = mipmap_size(texture_size, lod);
    let tile_size = constants.tile_size;
    let tiles = texture_mip_size / tile_size;
    let origin = vec2<f32>(textureLoad(page_table_texture, vec2<i32>(tex_coord * tiles), i32(lod)).xy * u32(tile_size));
    let factor = (tex_coord * texture_mip_size % tile_size) / tile_size;
    var uv = vec2<f32>(0.0, 0.0);
    uv.x = mix(origin.x, origin.x + tile_size, factor.x);
    uv.y = mix(origin.y, origin.y + tile_size, factor.y);
    uv = uv / physical_size;
    var color = textureSampleLevel(physical_texture, base_color_sampler, uv, 0.0);
    return color;
}

@vertex fn vs_main(vertex_in: VertexIn) -> VertexOutput {
    let mv = constants.view * constants.model;
    let mvp = constants.projection * mv;
    var result: VertexOutput;
    result.tex_coord = vertex_in.tex_coord;
    result.position = mvp * vec4<f32>(vertex_in.position, 1.0);
    result.vertex_color = vertex_in.vertex_color;
    result.normal = vertex_in.normal;
    result.frag_position = (constants.model * vec4<f32>(vertex_in.position, 1.0)).xyz;
    return result;
}

@fragment fn fs_main(vertex: VertexOutput) -> FragmentOutput {
    var fragment_output: FragmentOutput;
    if constants.is_enable_virtual_texture == 1 {
        if constants.is_virtual_diffuse_texture == 1 {
            let diffuse_color = virtual_texture_sample(vertex.tex_coord, constants.diffuse_texture_max_lod, constants.diffuse_texture_size);
            fragment_output.color = diffuse_color;
        } else {
            let diffuse_color = textureSample(diffuse_texture, base_color_sampler, vertex.tex_coord);
            fragment_output.color = diffuse_color;
        }
    } else {
        let diffuse_color = textureSample(diffuse_texture, base_color_sampler, vertex.tex_coord);
        let specular_color = textureSample(specular_texture, base_color_sampler, vertex.tex_coord);
        fragment_output.color = diffuse_color;
    }
    return fragment_output;
}
