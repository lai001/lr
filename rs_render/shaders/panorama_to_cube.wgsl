#include "constants.wgsl"
#include "sample_equirectangular.wgsl"

#ifndef TEXTURE_FORMAT
    #define TEXTURE_FORMAT rg11b10float
#endif

@group(0)
@binding(0)
var equirectangular_texture: texture_2d<f32>;

@group(0)
@binding(1)
var cube_map: texture_storage_2d_array<TEXTURE_FORMAT, write>;

@compute
@workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
   var cube_map_texture_dimensions = textureDimensions(cube_map);
   var sample_uv = (vec2<f32>(global_id.xy) + vec2(0.5)) / vec2<f32>(cube_map_texture_dimensions.xy) * 2.0 - 1.0;
   var sample_picker = get_sample_picker(global_id.z, sample_uv);
   var color = sample_equirectangular(equirectangular_texture, sample_picker, 0);
   textureStore(cube_map, vec2<i32>(global_id.xy), i32(global_id.z), color);
}