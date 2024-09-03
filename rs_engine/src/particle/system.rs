use super::emiter::ParticleEmiter;
use std::collections::HashMap;

pub struct ParticleSystem {
    pub name: String,
    pub emiters: HashMap<String, ParticleEmiter>,
    pub time: f32,
    is_finish: Option<bool>,
}

impl ParticleSystem {
    pub fn new(name: String) -> ParticleSystem {
        ParticleSystem {
            emiters: HashMap::new(),
            time: 0.0,
            name,
            is_finish: None,
        }
    }

    pub fn add_emiter(&mut self, emiter: ParticleEmiter) {
        let name = {
            match &emiter {
                ParticleEmiter::Spawn(emiter) => emiter.name.clone(),
            }
        };
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
                    ParticleEmiter::Spawn(emiter) => {
                        emiter.reset();
                    }
                }
            }
            self.is_finish = Some(true);
        } else {
            self.is_finish = Some(false);
        }
        for (_, emiter) in self.emiters.iter_mut() {
            match emiter {
                ParticleEmiter::Spawn(emiter) => {
                    emiter.tick(old_time, delta_time);
                }
            }
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

    pub fn get_is_finish(&self) -> bool {
        match self.is_finish {
            Some(is_finish) => is_finish,
            None => false,
        }
    }
}
