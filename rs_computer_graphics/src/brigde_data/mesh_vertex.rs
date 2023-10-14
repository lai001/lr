use type_layout::TypeLayout;

// #[repr(C)]
// #[derive(Clone, Copy, Debug)]
// pub struct MeshVertex {
//     pub position: glam::Vec3,
//     pub tex_coord: glam::Vec2,
//     pub vertex_color: glam::Vec4,
//     pub normal: glam::Vec3,
// }

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, TypeLayout)]
pub struct MeshVertex {
    pub vertex_color: glam::Vec4,
    pub position: glam::Vec3,
    pub normal: glam::Vec3,
    pub tangent: glam::Vec3,
    pub bitangent: glam::Vec3,
    pub tex_coord: glam::Vec2,
}
