use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EPropertyValueType {
    Texture(Option<url::Url>),
    Int(i32),
    Float(f32),
    String(String),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Quat(glam::Quat),
}
