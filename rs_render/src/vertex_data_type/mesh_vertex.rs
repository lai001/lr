use type_layout::TypeLayout;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, TypeLayout)]
pub struct MeshVertex0 {
    pub position: glam::Vec3,
    pub tex_coord: glam::Vec2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, TypeLayout)]
pub struct MeshVertex1 {
    pub vertex_color: glam::Vec4,
    pub normal: glam::Vec3,
    pub tangent: glam::Vec3,
    pub bitangent: glam::Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, TypeLayout)]
pub struct MeshVertex2 {
    pub bone_ids: glam::IVec4,
    pub bone_weights: glam::Vec4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, TypeLayout)]
pub struct MeshVertex3 {
    pub position: glam::Vec3,
    pub vertex_color: glam::Vec3,
}
