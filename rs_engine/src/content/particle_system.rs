use std::collections::HashMap;

use crate::url_extension::UrlExtension;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

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

impl Asset for ParticleSystem {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::ParticleSystem)
    }
}
