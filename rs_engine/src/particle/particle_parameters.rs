pub struct ParticleParameters {
    pub positions: Vec<glam::Vec3>,
    pub colors: Vec<glam::Vec4>,
    pub speeds: Vec<glam::Vec3>,
    pub velocities: Vec<glam::Vec3>,
    pub lifetimes: Vec<glam::Vec2>,
    pub is_alive: Vec<bool>,
}

impl ParticleParameters {
    pub fn new(len: usize) -> ParticleParameters {
        ParticleParameters {
            positions: vec![glam::Vec3::ZERO; len],
            colors: vec![glam::Vec4::ZERO; len],
            speeds: vec![glam::Vec3::ZERO; len],
            velocities: vec![glam::Vec3::ZERO; len],
            lifetimes: vec![glam::Vec2::ZERO; len],
            is_alive: vec![false; len],
        }
    }

    pub fn get_count(&self) -> usize {
        self.positions.len()
    }
}

#[derive(Debug, Clone)]
pub struct VelocityVariant {
    pub start: glam::Vec3,
    pub end: glam::Vec3,
}

#[derive(Debug, Clone)]
pub struct LifetimeVariant {
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, Clone)]
pub struct ColorVariant {
    pub start: glam::Vec4,
    pub end: glam::Vec4,
}

pub struct ParticleVariants {
    pub velocity_variants: Vec<VelocityVariant>,
    pub color_variants: Vec<ColorVariant>,
}

impl ParticleVariants {
    pub fn new(len: usize) -> ParticleVariants {
        ParticleVariants {
            velocity_variants: vec![
                VelocityVariant {
                    start: glam::Vec3::ZERO,
                    end: glam::Vec3::ZERO,
                };
                len
            ],
            color_variants: vec![
                ColorVariant {
                    start: glam::Vec4::ZERO,
                    end: glam::Vec4::ZERO
                };
                len
            ],
        }
    }

    pub fn get_count(&self) -> usize {
        self.velocity_variants.len()
    }
}
