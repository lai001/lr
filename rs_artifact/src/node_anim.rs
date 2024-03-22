use crate::{asset::Asset, default_url, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VectorKey {
    pub time: f64,
    pub value: glam::Vec3,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuatKey {
    pub time: f64,
    pub value: glam::Quat,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeAnim {
    pub name: String,
    pub id: uuid::Uuid,
    pub url: url::Url,
    pub position_keys: Vec<VectorKey>,
    pub scaling_keys: Vec<VectorKey>,
    pub rotation_keys: Vec<QuatKey>,
}

impl Asset for NodeAnim {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::NodeAnim
    }
}

impl Default for NodeAnim {
    fn default() -> Self {
        Self {
            position_keys: Default::default(),
            scaling_keys: Default::default(),
            rotation_keys: Default::default(),
            name: Default::default(),
            id: Default::default(),
            url: default_url().clone(),
        }
    }
}
