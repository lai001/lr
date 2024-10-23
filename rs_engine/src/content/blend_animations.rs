use crate::{skeleton_animation_provider::SkeletonAnimationBlendType, url_extension::UrlExtension};
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Channel {
    pub animation_url: url::Url,
    pub blend_type: SkeletonAnimationBlendType,
    pub time_range: std::ops::RangeInclusive<f32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlendAnimations {
    pub url: url::Url,
    pub channels: Vec<Channel>,
}

impl BlendAnimations {
    pub fn new(url: url::Url) -> BlendAnimations {
        BlendAnimations {
            url,
            channels: vec![],
        }
    }

    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }
}

impl Asset for BlendAnimations {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::BlendAnimations)
    }
}
