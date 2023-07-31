#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: glam::Vec3,
    _padding: u32,
    pub ambient: glam::Vec3,
    _padding1: u32,
    pub diffuse: glam::Vec3,
    _padding2: u32,
    pub specular: glam::Vec3,
    _padding3: u32,
}

impl DirectionalLight {
    pub fn default() -> DirectionalLight {
        DirectionalLight {
            direction: glam::Vec3::new(-0.2, -1.0, -0.3),
            ambient: glam::vec3(0.2, 0.2, 0.2),
            diffuse: glam::vec3(0.2, 0.2, 0.2),
            specular: glam::vec3(0.2, 0.2, 0.2),
            _padding: 0,
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: glam::Vec3,
    _padding: u32,
    pub ambient: glam::Vec3,
    _padding1: u32,
    pub diffuse: glam::Vec3,
    _padding2: u32,
    pub specular: glam::Vec3,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
    _padding3: [u32; 2],
}

impl PointLight {
    pub fn default() -> PointLight {
        PointLight {
            position: glam::Vec3::ZERO,
            ambient: glam::Vec3::ZERO,
            diffuse: glam::Vec3::ZERO,
            specular: glam::Vec3::ZERO,
            constant: 0.0,
            linear: 0.0,
            quadratic: 0.0,
            _padding: 0,
            _padding1: 0,
            _padding2: 0,
            _padding3: [0, 0],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SpotLight {
    pub position: glam::Vec3,
    _padding: u32,
    pub direction: glam::Vec3,
    _padding1: u32,
    pub ambient: glam::Vec3,
    _padding2: u32,
    pub diffuse: glam::Vec3,
    _padding3: u32,
    pub specular: glam::Vec3,
    pub cut_off: f32,
    pub outer_cut_off: f32,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl SpotLight {
    pub fn default() -> SpotLight {
        SpotLight {
            position: glam::Vec3::ZERO,
            direction: glam::Vec3::ZERO,
            ambient: glam::Vec3::ZERO,
            diffuse: glam::Vec3::ZERO,
            specular: glam::Vec3::ZERO,
            cut_off: 0.0,
            outer_cut_off: 0.0,
            constant: 0.0,
            linear: 0.0,
            quadratic: 0.0,
            _padding: 0,
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
        }
    }
}
