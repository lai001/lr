use type_layout::TypeLayout;

pub const INVALID_BONE: i32 = -1;

#[repr(C)]
#[derive(Clone, Copy, Debug, TypeLayout)]
pub struct SkinMeshVertex {
    pub vertex_color: glam::Vec4,
    pub position: glam::Vec3,
    pub normal: glam::Vec3,
    pub tangent: glam::Vec3,
    pub bitangent: glam::Vec3,
    pub tex_coord: glam::Vec2,
    pub bones: [i32; 4],
    pub weights: [f32; 4],
}
