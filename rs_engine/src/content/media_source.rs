use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaSource {
    pub url: url::Url,
    pub asset_url: url::Url,
}

impl Asset for MediaSource {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::MediaSource)
    }
}
