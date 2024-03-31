use rs_engine::url_extension::UrlExtension;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkeletonMesh {
    pub name: String,
    pub url: url::Url,
    pub skeleton_url: url::Url,
    pub asset_reference: String,
}

impl SkeletonMesh {
    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }
}
