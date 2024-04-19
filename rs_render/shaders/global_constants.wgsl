#ifndef GLOBAL_CONSTANTS
#define GLOBAL_CONSTANTS

struct GlobalConstants {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    physical_texture_size: f32,
    tile_size: f32,
    is_enable_virtual_texture: i32,
    scene_factor: f32,
    feedback_bias: f32,
};

#endif