#ifndef OUTPUT_TEXTURE_FORMAT
    #define OUTPUT_TEXTURE_FORMAT rgba8unorm
#endif

@group(0) @binding(0) var input_texture: texture_depth_2d;

@group(0) @binding(1) var output_texture: texture_storage_2d<OUTPUT_TEXTURE_FORMAT, write>;

@compute
@workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var color = textureLoad(input_texture, vec2<i32>(global_id.xy), i32(global_id.z));
    textureStore(output_texture, vec2<i32>(global_id.xy), vec4<f32>(vec3<f32>(color), 1.0));
}