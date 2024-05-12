use crate::global_shaders::skeleton_shading::NUM_MAX_BONE;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Constants {
    pub model: glam::Mat4,
    pub id: u32,
    _pad_0: u32,
    _pad_1: u32,
    _pad_2: u32,
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
