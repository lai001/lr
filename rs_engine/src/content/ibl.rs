use crate::url_extension::UrlExtension;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use rs_render::bake_info::BakeInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct IBL {
    pub url: url::Url,
    pub bake_info: BakeInfo,
    pub ibl_baking_url: Option<url::Url>,
    pub image_reference: Option<PathBuf>,
}

impl Asset for IBL {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::IBL)
    }
}

impl IBL {
    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    pub fn new(url: url::Url) -> IBL {
        IBL {
            url,
            bake_info: Default::default(),
            ibl_baking_url: None,
            image_reference: None,
        }
    }
}
