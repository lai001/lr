use crate::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Material {
    pub url: url::Url,
    pub code: String,
}

impl Asset for Material {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Material
    }
}
