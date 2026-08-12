use crate::url_extension::UrlExtension;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParticleSpawnEmiterPros {
    pub rate: f32,
    pub count: usize,
    pub time_range: glam::Vec2,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EParticleEmiterType {
    Spawn(ParticleSpawnEmiterPros),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParticleSystem {
    pub url: url::Url,
    pub max_particles: usize,
    pub emiters: HashMap<String, EParticleEmiterType>,
}
crate::impl_content!(ParticleSystem);

impl ParticleSystem {
    pub fn new(url: url::Url) -> ParticleSystem {
        ParticleSystem {
            url,
            max_particles: 500,
            emiters: HashMap::new(),
        }
    }

    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    pub fn new_template_instance(&self, name: String) -> crate::particle::system::ParticleSystem {
        crate::particle::system::ParticleSystem::new(name)
    }
}
