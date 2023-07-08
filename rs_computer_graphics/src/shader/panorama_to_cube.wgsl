@group(0)
@binding(0)
var equirectangular_map: texture_2d<f32>;

@group(0)
@binding(1)
var cube_map: texture_storage_2d_array<rgba32float, write>;

const M_PI: f32 = 3.1415; 

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
   var cube_map_texture_dimensions = textureDimensions(cube_map);
   var eq_map_texture_dimensions = textureDimensions(equirectangular_map);

   var sample_picker = vec3<f32>(0.0);

   var sample_uv = vec2<f32>(global_id.xy) / vec2<f32>(cube_map_texture_dimensions.xy) * 2.0 - 1.0;

   if global_id.z == u32(0) {
      sample_picker = vec3<f32>(1.0, sample_uv.y, -sample_uv.x);
   } else if global_id.z == u32(1) {
      sample_picker = vec3<f32>(-1.0, sample_uv.y, sample_uv.x);
   } else if global_id.z == u32(2) {
      sample_picker = vec3<f32>(sample_uv.x, 1.0, -sample_uv.y);
   } else if global_id.z == u32(3) {
      sample_picker = vec3<f32>(sample_uv.x, -1.0, sample_uv.y);
   } else if global_id.z == u32(4) {
      sample_picker = vec3<f32>(sample_uv.x, sample_uv.y, 1.0);
   } else if global_id.z == u32(5) {
      sample_picker = vec3<f32>(-sample_uv.x, sample_uv.y, -1.0);
   }
   sample_picker = normalize(sample_picker);
   var uv_x: f32 = ((atan2(sample_picker.z, sample_picker.x) + M_PI) / (M_PI * 2.0));
   var uv_y: f32 = (acos(sample_picker.y) / M_PI);
   var uv = vec2<f32>(uv_x, uv_y);
   var uv_i32 = vec2<i32>(i32(uv.x * f32(eq_map_texture_dimensions.x)), i32(uv.y * f32(eq_map_texture_dimensions.y)));

   var color = textureLoad(equirectangular_map, uv_i32, i32(0));
   textureStore(cube_map, vec2<i32>(global_id.xy), i32(global_id.z), color);
}