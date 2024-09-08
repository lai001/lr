use super::particle_parameters::{
    ColorVariant, ParticleParameters, ParticleVariants, VelocityVariant,
};

pub struct ParticleSpawnEmiter {
    pub spawn_rate: f32,
    pub count_per_spawn: usize,
    pub time_range: glam::Vec2,
    emit_count: usize,
    pub name: String,
    pub particle_parameters: ParticleParameters,
    pub particle_variants: ParticleVariants,
    pub index: usize,
    pub spawn_at: glam::Vec3,
}

impl ParticleSpawnEmiter {
    pub fn new(
        name: String,
        rate: f32,
        count: usize,
        time_range: glam::Vec2,
        len: usize,
        spawn_at: glam::Vec3,
    ) -> ParticleSpawnEmiter {
        assert!(time_range.y > time_range.x);
        assert!(rate > 0.0);
        ParticleSpawnEmiter {
            spawn_rate: rate,
            count_per_spawn: count,
            time_range,
            emit_count: 0,
            name,
            index: 0,
            particle_parameters: ParticleParameters::new(len),
            particle_variants: ParticleVariants::new(len),
            spawn_at,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
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
        let previous = (time / self.spawn_rate - self.time_range.x % delta_time).floor() as usize;
        if previous == self.emit_count {
            return can_emit_count;
        }
        can_emit_count = (particle_count - index).min(self.count_per_spawn);
        can_emit_count
    }

    pub fn emit(&mut self, time: f32, delta_time: f32) -> usize {
        let particle_count = { self.particle_parameters.positions.len() };
        let index = self.index;
        let can_emit_count: usize = self.can_emit_count(time, delta_time, index, particle_count);
        let particle_parameters = &mut self.particle_parameters;
        let particle_variants = &mut self.particle_variants;

        if can_emit_count == 0 {
            return can_emit_count;
        }
        let range = index..index + can_emit_count;

        for position in &mut particle_parameters.positions[range.clone()] {
            *position = self.spawn_at;
        }
        for is_alive in &mut particle_parameters.is_alive[range.clone()] {
            *is_alive = true;
        }
        for color in &mut particle_parameters.colors[range.clone()] {
            *color = glam::Vec4::ZERO;
        }
        for speed in &mut particle_parameters.speeds[range.clone()] {
            *speed = glam::Vec3::ZERO;
        }
        for velocity in &mut particle_parameters.velocities[range.clone()] {
            *velocity = glam::Vec3::ZERO;
        }
        for lifetime in &mut particle_parameters.lifetimes[range.clone()] {
            let duration = Self::random_duration();
            *lifetime = glam::vec2(time, duration);
        }

        for color_variant in &mut particle_variants.color_variants[range.clone()] {
            *color_variant = ColorVariant {
                start: Self::random_color(),
                end: Self::random_color(),
            };
        }

        for velocity_variant in &mut particle_variants.velocity_variants[range.clone()] {
            *velocity_variant = VelocityVariant {
                start: Self::random_velocity(),
                end: Self::random_velocity(),
            };
        }

        self.emit_count += 1;
        can_emit_count
    }

    pub fn tick(&mut self, time: f32, delta_time: f32) {
        let emit_count = self.emit(time, delta_time);

        self.index += emit_count;
        self.index %= self.particle_parameters.get_count();

        for i in 0..self.particle_parameters.get_count() {
            let lifetime = self.particle_parameters.lifetimes[i];
            let is_alive = (lifetime.x..lifetime.x + lifetime.y).contains(&time);
            self.particle_parameters.is_alive[i] = is_alive;
            if !is_alive {
                continue;
            }
            let alpha = (time - lifetime.x) / lifetime.y;
            let color_variant = &self.particle_variants.color_variants[i];
            self.particle_parameters.colors[i] = color_variant.start.lerp(color_variant.end, alpha);
            let velocity_variant = &self.particle_variants.velocity_variants[i];
            let velocity = velocity_variant.start.lerp(velocity_variant.end, alpha);
            let speed = self.particle_parameters.speeds[i] + velocity * delta_time;
            self.particle_parameters.speeds[i] = speed;
            let distance = speed * delta_time;
            self.particle_parameters.positions[i] += distance;
        }
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

    pub fn get_parameters(&self) -> Vec<(glam::Vec3, glam::Vec4)> {
        let position_colors = (0..self.particle_parameters.get_count())
            .filter_map(|i| {
                let is_alive = self.particle_parameters.is_alive[i];
                if is_alive {
                    Some((
                        self.particle_parameters.positions[i].clone(),
                        self.particle_parameters.colors[i].clone(),
                    ))
                } else {
                    None
                }
            })
            .collect();
        position_colors
    }
}

pub enum ParticleEmiter {
    Spawn(ParticleSpawnEmiter),
}

impl ParticleEmiter {
    pub fn get_name(&self) -> &str {
        match self {
            ParticleEmiter::Spawn(x) => &x.name,
        }
    }
}
