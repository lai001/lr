use rs_engine::url_extension::UrlExtension;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Skeleton {
    pub url: url::Url,
    pub asset_reference: String,
    pub root_bone: String,
}

impl Skeleton {
    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }
}
