#[repr(C)]
#[derive(Clone, Copy)]
pub struct Image2DVertex {
    pub pos: glam::Vec2,
    pub uv: glam::Vec2,
}
