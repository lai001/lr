#include "common.wgsl"

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