#ifndef VIRTUAL_TEXTURE_SLANG
#define VIRTUAL_TEXTURE_SLANG

#include "common.slang"

float4 virtual_texture_sample(float2 tex_coord,
                              uint max_lod,
                              float2 texture_size,
                              float2 physical_size,
                              float tile_size,
                              Texture2D<uint4> page_table_texture,
                              Texture2D physical_texture,
                              SamplerState base_color_sampler)
{
    float lod = min(uint(mipmap_level(tex_coord, float2(physical_size.x, physical_size.y))), max_lod);
    float2 texture_mip_size = mipmap_size(texture_size, uint(lod));
    float2 tiles = texture_mip_size / tile_size;
    uint2 origin_uv = page_table_texture.Load(int3(int2(tex_coord * tiles), int(lod))).xy * uint(tile_size);
    float2 factor = (frac(tex_coord * texture_mip_size / tile_size));
    float2 uv;
    uv.x = lerp(float(origin_uv.x), float(origin_uv.x) + tile_size, factor.x);
    uv.y = lerp(float(origin_uv.y), float(origin_uv.y) + tile_size, factor.y);
    uv = uv / physical_size;
    float4 color = physical_texture.SampleLevel(base_color_sampler, uv, 0.0);
    return color;
}

#endif
