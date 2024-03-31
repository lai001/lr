use crate::asset::Asset;
use crate::resource_type::EResourceType;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Skeleton {
    pub name: String,
    pub url: url::Url,
}

impl Asset for Skeleton {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Skeleton
    }
}
