use rs_engine::url_extension::UrlExtension;
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
