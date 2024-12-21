#[repr(i32)]
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum EDebugShadingType {
    None = 0,
    BaseColor = 1,
    Metallic = 2,
    Roughness = 3,
    Normal = 4,
    VertexColor0 = 5,
    Shadow = 6,
}

impl EDebugShadingType {
    pub fn all_types() -> Vec<EDebugShadingType> {
        vec![
            EDebugShadingType::None,
            EDebugShadingType::BaseColor,
            EDebugShadingType::Metallic,
            EDebugShadingType::Roughness,
            EDebugShadingType::Normal,
            EDebugShadingType::VertexColor0,
            EDebugShadingType::Shadow,
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CameraFrustum {
    pub near_0: glam::Vec3,
    _pad_0: i32,
    pub near_1: glam::Vec3,
    _pad_1: i32,
    pub near_2: glam::Vec3,
    _pad_2: i32,
    pub near_3: glam::Vec3,
    _pad_3: i32,
    pub far_0: glam::Vec3,
    _pad_4: i32,
    pub far_1: glam::Vec3,
    _pad_5: i32,
    pub far_2: glam::Vec3,
    _pad_6: i32,
    pub far_3: glam::Vec3,
    _pad_7: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Constants {
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
    pub view_projection: glam::Mat4,
    pub light_space_matrix: glam::Mat4,
    pub view_position: glam::Vec3,
    pub physical_texture_size: f32,
    pub tile_size: f32,
    pub is_enable_virtual_texture: i32,
    pub scene_factor: f32,
    pub feedback_bias: f32,
    debug_shading: i32,
    pub time: f32,
    _pad_0: [i32; 2],
    pub camera_frustum: CameraFrustum,
}

impl Constants {
    pub fn set_shading_type(&mut self, ty: EDebugShadingType) {
        self.debug_shading = ty as i32;
    }

    pub fn get_shading_type(&mut self) -> EDebugShadingType {
        unsafe { ::std::mem::transmute(self.debug_shading) }
    }
}
