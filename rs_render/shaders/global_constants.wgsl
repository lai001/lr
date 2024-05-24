#ifndef GLOBAL_CONSTANTS
#define GLOBAL_CONSTANTS

const DEBUG_SHADING_TYPE_NONE: i32 = 0;
const DEBUG_SHADING_TYPE_BASE_COLOR: i32 = 1;
const DEBUG_SHADING_TYPE_METALLIC: i32 = 2;
const DEBUG_SHADING_TYPE_ROUGHNESS: i32 = 3;
const DEBUG_SHADING_TYPE_NORMAL: i32 = 4;
const DEBUG_SHADING_TYPE_VERTEX_COLOR_0: i32 = 5;

struct GlobalConstants {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    view_position: vec3<f32>,
    physical_texture_size: f32,
    tile_size: f32,
    is_enable_virtual_texture: i32,
    scene_factor: f32,
    feedback_bias: f32,
    debug_shading: i32,
};

#endif