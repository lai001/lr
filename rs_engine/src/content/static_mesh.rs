use crate::url_extension::UrlExtension;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticMesh {
    pub asset_reference_name: String,
    pub url: url::Url,
    pub asset_reference_relative_path: String,
}

impl StaticMesh {
    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }
}

impl Asset for StaticMesh {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::StaticMesh)
    }
}
