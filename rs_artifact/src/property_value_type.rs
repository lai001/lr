use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EPropertyValueType {
    Texture(Option<url::Url>),
    Int(i32),
    Float(f32),
    String(String),
}
