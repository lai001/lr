#[repr(C)]
#[derive(Clone, Copy)]
pub struct MeshVertex {
    pub position: glam::Vec3,
    pub tex_coord: glam::Vec2,
    pub vertex_color: glam::Vec4,
    pub normal: glam::Vec3,
}
