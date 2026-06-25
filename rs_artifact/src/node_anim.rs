use serde::{Deserialize, Serialize};

pub const CUBIC_SPLINE_CONTROL_KEYS_NUM: usize = 3;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EVectorAnimInterpolation {
    Step(glam::Vec3),
    Linear(glam::Vec3),
    SphericalLinear(glam::Vec3),
    CubicSpline([glam::Vec3; CUBIC_SPLINE_CONTROL_KEYS_NUM]),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EQuatAnimInterpolation {
    Step(glam::Quat),
    Linear(glam::Quat),
    SphericalLinear(glam::Quat),
    CubicSpline([glam::Quat; CUBIC_SPLINE_CONTROL_KEYS_NUM]),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VectorKey {
    pub time: f64,
    pub value: glam::Vec3,
    pub interpolation: EVectorAnimInterpolation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuatKey {
    pub time: f64,
    pub value: glam::Quat,
    pub interpolation: EQuatAnimInterpolation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeAnim {
    pub node: String,
    pub position_keys: Vec<VectorKey>,
    pub scaling_keys: Vec<VectorKey>,
    pub rotation_keys: Vec<QuatKey>,
}
