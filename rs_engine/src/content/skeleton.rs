use crate::url_extension::UrlExtension;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Skeleton {
    pub url: url::Url,
    pub asset_url: url::Url,
}

impl Skeleton {
    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    pub fn get_root_bone(&self) -> String {
        let root_bone = self
            .asset_url
            .query_pairs()
            .find(|x| x.0 == "root_bone")
            .unwrap()
            .1
            .to_string();
        root_bone
    }

    pub fn get_relative_path(&self) -> String {
        format!(
            "{}{}",
            self.asset_url.domain().unwrap(),
            self.asset_url.path()
        )
    }

    pub fn make_asset_url(relative_path: &str, root_bone: &str) -> url::Url {
        url::Url::parse(&format!(
            "asset://{}?root_bone={}",
            relative_path, root_bone
        ))
        .unwrap()
    }
}

impl Asset for Skeleton {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::Skeleton)
    }
}

#[test]
fn test() {
    let url = url::Url::parse("asset://model/b/test.glb?root_bone=/root/armature").unwrap();
    println!("domain: {:?}", url.domain());
    println!("host: {:?}", url.host());
    println!("query: {:?}", url.query());
    println!("path: {:?}", url.path());
    println!("scheme: {:?}", url.scheme());
    println!("{}{}", url.domain().unwrap(), url.path());
    for p in url.query_pairs() {
        println!("{:?}", p);
    }
}
