use crate::global_shaders::skeleton_shading::NUM_MAX_BONE;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    pub model: glam::Mat4,
    pub id: u32,
    _pad_0: u32,
    _pad_1: u32,
    _pad_2: u32,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            model: glam::Mat4::IDENTITY,
            id: 0,
            _pad_0: 0,
            _pad_1: 0,
            _pad_2: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkinConstants {
    pub bones: [glam::Mat4; NUM_MAX_BONE],
}

impl Default for SkinConstants {
    fn default() -> Self {
        Self {
            bones: [glam::Mat4::IDENTITY; NUM_MAX_BONE],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VirtualTextureConstants {
    pub virtual_texture_size: glam::Vec2,
    pub virtual_texture_max_lod: u32,
    _pad_0: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct IBLConstants {
    pub sample_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SDF2DConstants {
    pub channel: i32,
    pub threshold: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JFAConstants {
    pub step: glam::Vec2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PreFilterConstants {
    pub roughness: f32,
    pub sample_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MeshViewConstants {
    pub model: glam::Mat4,
}
