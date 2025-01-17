use crate::{build_asset_url, url_extension::UrlExtension};
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetInfo {
    pub relative_path: PathBuf,
    pub path: String,
}

impl AssetInfo {
    pub fn get_url(&self) -> url::Url {
        build_asset_url(format!(
            "{}?path={}",
            self.relative_path.as_os_str().to_string_lossy().to_string(),
            self.path
        ))
        .unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticMesh {
    // pub asset_reference_name: String,
    pub url: url::Url,
    // pub asset_reference_relative_path: String,
    pub asset_info: AssetInfo,
    pub is_enable_multiresolution: bool,
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
