#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Constants {
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
    pub view_projection: glam::Mat4,
    pub view_position: glam::Vec3,
    pub physical_texture_size: f32,
    pub tile_size: f32,
    pub is_enable_virtual_texture: i32,
    pub scene_factor: f32,
    pub feedback_bias: f32,
}
