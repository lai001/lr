use crate::{build_asset_url, url_extension::UrlExtension};
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
crate::impl_content!(StaticMesh);

impl StaticMesh {
    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }
}
