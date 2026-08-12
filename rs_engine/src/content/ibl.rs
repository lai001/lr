use crate::url_extension::UrlExtension;
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

crate::impl_content!(IBL);

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
