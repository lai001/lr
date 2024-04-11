use crate::{asset::Asset, node_anim::NodeAnim, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkeletonAnimation {
    pub name: String,
    pub url: url::Url,
    pub duration: f64,
    pub ticks_per_second: f64,
    pub channels: Vec<NodeAnim>,
}

impl Asset for SkeletonAnimation {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::SkeletonAnimation
    }
}
