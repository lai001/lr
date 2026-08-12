use crate::node_anim::NodeAnim;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkeletonAnimation {
    pub name: String,
    pub url: url::Url,
    pub duration: f64,
    pub ticks_per_second: f64,
    pub channels: Vec<NodeAnim>,
}

impl SkeletonAnimation {
    pub fn duration_as_secs_f32(&self) -> f32 {
        (self.duration / self.ticks_per_second) as f32
    }
}

crate::impl_asset!(SkeletonAnimation);
