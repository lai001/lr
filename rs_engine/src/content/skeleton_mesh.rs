use crate::url_extension::UrlExtension;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkeletonMesh {
    pub url: url::Url,
    pub skeleton_url: url::Url,
    pub asset_url: url::Url,
}

impl SkeletonMesh {
    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    pub fn get_skeleton_mesh_name(&self) -> String {
        self.asset_url
            .query_pairs()
            .find(|x| x.0 == "skeleton_mesh_name")
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

    pub fn make_asset_url(relative_path: &str, skeleton_mesh_name: &str) -> url::Url {
        url::Url::parse(&format!(
            "asset://{}?skeleton_mesh_name={}",
            relative_path, skeleton_mesh_name
        ))
        .unwrap()
    }
}

impl Asset for SkeletonMesh {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::SkeletonMesh)
    }
}
