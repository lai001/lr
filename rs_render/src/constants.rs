use crate::global_shaders::skeleton_shading::NUM_MAX_BONE;

pub const MAX_POINT_LIGHTS_NUM: u32 = 2;
pub const MAX_SPOT_LIGHTS_NUM: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    pub model: glam::Mat4,
    pub id: u32,
    _pad_0: u32,
    _pad_1: u32,
    _pad_2: u32,
}

impl Constants {
    pub fn new(model: glam::Mat4, id: u32) -> Constants {
        Self {
            model,
            id,
            _pad_0: 0,
            _pad_1: 0,
            _pad_2: 0,
        }
    }
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: glam::Vec3,
    _pad_0: u32,
    pub ambient: glam::Vec3,
    _pad_1: u32,
    pub diffuse: glam::Vec3,
    _pad_2: u32,
    pub specular: glam::Vec3,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
    _pad_3: u32,
    _pad_4: u32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            _pad_0: 0,
            ambient: glam::Vec3::ONE,
            _pad_1: 0,
            diffuse: glam::Vec3::ONE,
            _pad_2: 0,
            specular: glam::Vec3::ONE,
            _pad_3: 0,
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
            _pad_4: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct PointLights {
    pub lights: [PointLight; MAX_POINT_LIGHTS_NUM as usize],
    pub available: u32,
    _pad: [u32; 3],
}

impl Default for PointLights {
    fn default() -> Self {
        Self {
            lights: [PointLight::default(); MAX_POINT_LIGHTS_NUM as usize],
            available: 0,
            _pad: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SpotLight {
    pub light: PointLight,
    pub direction: glam::Vec3,
    pub cut_off: f32,
    pub outer_cut_off: f32,
    _pad: [u32; 3],
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            light: PointLight::default(),
            cut_off: 12.5_f32.to_radians(),
            outer_cut_off: 17.5_f32.to_radians(),
            direction: glam::Vec3::Z,
            _pad: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct SpotLights {
    pub lights: [SpotLight; MAX_SPOT_LIGHTS_NUM as usize],
    pub available: u32,
    _pad: [u32; 3],
}

impl Default for SpotLights {
    fn default() -> Self {
        Self {
            lights: [SpotLight::default(); MAX_SPOT_LIGHTS_NUM as usize],
            available: 0,
            _pad: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ClusterLightIndex {
    pub offset: u32,
    pub count: u32,
}
