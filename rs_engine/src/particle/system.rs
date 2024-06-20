use super::{
    emiter::ParticleEmiter,
    particle_parameters::{ParticleParameters, ParticleVariants},
};
use std::collections::HashMap;

pub struct ParticleSystem {
    pub particle_parameters: ParticleParameters,
    pub particle_variants: ParticleVariants,
    pub emiters: HashMap<String, ParticleEmiter>,
    pub time: f32,
    pub index: usize,
}

impl ParticleSystem {
    pub fn new(len: usize) -> ParticleSystem {
        ParticleSystem {
            particle_parameters: ParticleParameters::new(len),
            emiters: HashMap::new(),
            time: 0.0,
            particle_variants: ParticleVariants::new(len),
            index: 0,
        }
    }

    pub fn add_emiter(&mut self, name: String, emiter: ParticleEmiter) {
        self.emiters.insert(name, emiter);
    }

    pub fn remove_emiter(&mut self, name: impl AsRef<str>) {
        self.emiters.remove(name.as_ref());
    }

    pub fn tick(&mut self, delta_time: f32) {
        let total_time = self.get_total_time();
        if (total_time - 0.0).abs() <= f32::EPSILON {
            return;
        }

        let mut old_time = self.time;
        self.time += delta_time;

        if self.time >= total_time {
            self.time %= total_time;
            old_time = self.time;
            for (_, emiter) in self.emiters.iter_mut() {
                match emiter {
                    ParticleEmiter::Spawn(emiter) => emiter.reset(),
                }
            }
        }
        for (_, emiter) in self.emiters.iter_mut() {
            match emiter {
                ParticleEmiter::Spawn(emiter) => {
                    let emit_count = emiter.emit(
                        old_time,
                        delta_time,
                        &mut self.particle_parameters,
                        &mut self.particle_variants,
                        self.index,
                    );

                    self.index += emit_count;
                    self.index %= self.particle_parameters.get_count();
                }
            }
        }

        for i in 0..self.particle_parameters.get_count() {
            let lifetime = self.particle_parameters.lifetimes[i];
            let is_alive = (lifetime.x..lifetime.y).contains(&old_time);
            self.particle_parameters.is_alive[i] = is_alive;
            if !is_alive {
                continue;
            }
            let alpha = (old_time - lifetime.x) / (lifetime.y - lifetime.x);
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

    pub fn get_total_time(&self) -> f32 {
        let mut time: f32 = 0.0;
        for (_, emiter) in &self.emiters {
            match emiter {
                ParticleEmiter::Spawn(emiter) => {
                    time = time.max(emiter.time_range.y);
                }
            }
        }
        time
    }
}
