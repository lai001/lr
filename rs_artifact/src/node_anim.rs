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
    pub node: String,
    pub position_keys: Vec<VectorKey>,
    pub scaling_keys: Vec<VectorKey>,
    pub rotation_keys: Vec<QuatKey>,
}
