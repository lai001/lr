use super::particle_parameters::{
    ColorVariant, ParticleParameters, ParticleVariants, VelocityVariant,
};

pub struct ParticleSpawnEmiter {
    pub rate: f32,
    pub count: usize,
    pub time_range: glam::Vec2,
    emit_count: usize,
}

impl ParticleSpawnEmiter {
    pub fn new(rate: f32, count: usize, time_range: glam::Vec2) -> ParticleSpawnEmiter {
        assert!(time_range.y > time_range.x);
        ParticleSpawnEmiter {
            rate,
            count,
            time_range,
            emit_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.emit_count = 0;
    }

    pub fn can_emit_count(
        &self,
        time: f32,
        delta_time: f32,
        index: usize,
        particle_count: usize,
    ) -> usize {
        let mut can_emit_count: usize = 0;
        if !(self.time_range.x..self.time_range.y).contains(&time) {
            return can_emit_count;
        }
        if particle_count == 0 || index >= particle_count {
            return can_emit_count;
        }

        let previous = (time - self.time_range.x % delta_time).floor() as usize;
        if previous == self.emit_count {
            return can_emit_count;
        }

        can_emit_count = (particle_count - index).min(self.count);
        can_emit_count
    }

    pub fn emit(
        &mut self,
        time: f32,
        delta_time: f32,
        particle_parameters: &mut ParticleParameters,
        particle_variants: &mut ParticleVariants,
        index: usize,
    ) -> usize {
        let can_emit_count: usize =
            self.can_emit_count(time, delta_time, index, particle_parameters.positions.len());

        if can_emit_count == 0 {
            return can_emit_count;
        }

        for position in &mut particle_parameters.positions[index..index + can_emit_count] {
            *position = glam::Vec3::ZERO;
        }
        for is_alive in &mut particle_parameters.is_alive[index..index + can_emit_count] {
            *is_alive = true;
        }
        for color in &mut particle_parameters.colors[index..index + can_emit_count] {
            *color = glam::Vec4::ZERO;
        }
        for speed in &mut particle_parameters.speeds[index..index + can_emit_count] {
            *speed = glam::Vec3::ZERO;
        }
        for velocity in &mut particle_parameters.velocities[index..index + can_emit_count] {
            *velocity = glam::Vec3::ZERO;
        }
        for lifetime in &mut particle_parameters.lifetimes[index..index + can_emit_count] {
            let duration = Self::random_duration();
            *lifetime = glam::vec2(time, duration);
        }

        for color_variant in &mut particle_variants.color_variants[index..index + can_emit_count] {
            *color_variant = ColorVariant {
                start: Self::random_color(),
                end: Self::random_color(),
            };
        }

        for velocity_variant in
            &mut particle_variants.velocity_variants[index..index + can_emit_count]
        {
            *velocity_variant = VelocityVariant {
                start: Self::random_velocity(),
                end: Self::random_velocity(),
            };
        }

        self.emit_count += 1;
        can_emit_count
    }

    fn random_duration() -> f32 {
        let x: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..10.0);
        x
    }

    fn random_color() -> glam::Vec4 {
        let x: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
        let y: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
        let z: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
        glam::vec4(x, y, z, 1.0)
    }

    fn random_velocity() -> glam::Vec3 {
        let x: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), -5.0..5.0);
        let y: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), -5.0..5.0);
        let z: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), -5.0..5.0);
        glam::vec3(x, y, z)
    }
}

pub enum ParticleEmiter {
    Spawn(ParticleSpawnEmiter),
}
