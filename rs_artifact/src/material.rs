use crate::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct TextureBinding {
    pub group: usize,
    pub binding: usize,
    pub texture_url: url::Url,
}

impl TextureBinding {
    pub fn get_texture_bind_name(&self) -> String {
        format!("_texture_{}_{}", self.group, self.binding)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Material {
    pub url: url::Url,
    pub code: String,
    pub map_texture_names: HashSet<TextureBinding>,
}

impl Asset for Material {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Material
    }
}
