use crate::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShaderSourceCode {
    pub name: String,
    pub id: uuid::Uuid,
    pub url: url::Url,
    pub code: String,
}

impl Asset for ShaderSourceCode {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::ShaderSourceCode
    }
}
