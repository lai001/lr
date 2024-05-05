use crate::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IBLBaking {
    pub name: String,
    pub url: url::Url,
    pub brdf_data: Vec<u8>,
    pub pre_filter_data: Vec<u8>,
    pub irradiance_data: Vec<u8>,
}

impl Asset for IBLBaking {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::IBLBaking
    }
}
