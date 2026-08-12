use crate::{skeleton_animation_provider::SkeletonAnimationBlendType, url_extension::UrlExtension};
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

crate::impl_content!(BlendAnimations);

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
