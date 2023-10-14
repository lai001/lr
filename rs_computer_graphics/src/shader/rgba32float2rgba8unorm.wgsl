@group(0)
@binding(0)
var source_texture: texture_storage_2d<rgba32float, read>;

@group(0)
@binding(1)
var target_texture: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
   var source_color = textureLoad(source_texture, vec2<i32>(global_id.xy));
   source_color = clamp(source_color, vec4<f32>(0.0), vec4<f32>(1.0));
   source_color = source_color * vec4<f32>(255.0);
   textureStore(target_texture, vec2<i32>(global_id.xy), source_color);
}