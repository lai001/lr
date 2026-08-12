use crate::{build_asset_url, url_extension::UrlExtension};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkeletonAnimation {
    pub url: url::Url,
    pub asset_url: url::Url,
}
crate::impl_content!(SkeletonAnimation);

impl SkeletonAnimation {
    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    pub fn get_animation_name(&self) -> String {
        self.asset_url
            .query_pairs()
            .find(|x| x.0 == "animation_name")
            .unwrap()
            .1
            .to_string()
    }

    pub fn get_relative_path(&self) -> String {
        format!(
            "{}{}",
            self.asset_url.domain().unwrap(),
            self.asset_url.path()
        )
    }

    pub fn make_asset_url(relative_path: &str, animation_name: &str) -> url::Url {
        build_asset_url(format!(
            "{}?animation_name={}",
            relative_path, animation_name
        ))
        .unwrap()
    }
}
