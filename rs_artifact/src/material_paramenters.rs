use serde::{Deserialize, Serialize};

pub trait WgslMemoryLayout {
    fn align_of(&self) -> usize;
    fn size_of(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BaseDataValueType {
    F32(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
}

impl BaseDataValueType {
    pub fn all_type_names() -> Vec<&'static str> {
        vec!["F32", "Vec2", "Vec3", "Vec4"]
    }

    pub fn name(&self) -> &'static str {
        match self {
            BaseDataValueType::F32(_) => "F32",
            BaseDataValueType::Vec2(_) => "Vec2",
            BaseDataValueType::Vec3(_) => "Vec3",
            BaseDataValueType::Vec4(_) => "Vec4",
        }
    }

    pub fn default_value(name: &str) -> Option<BaseDataValueType> {
        match name {
            "F32" => Some(BaseDataValueType::F32(0.0)),
            "Vec2" => Some(BaseDataValueType::Vec2(Default::default())),
            "Vec3" => Some(BaseDataValueType::Vec3(Default::default())),
            "Vec4" => Some(BaseDataValueType::Vec4(Default::default())),
            _ => None,
        }
    }
}

impl WgslMemoryLayout for BaseDataValueType {
    fn align_of(&self) -> usize {
        match self {
            BaseDataValueType::F32(_) => 4,
            BaseDataValueType::Vec2(_) => 8,
            BaseDataValueType::Vec3(_) => 16,
            BaseDataValueType::Vec4(_) => 16,
        }
    }

    fn size_of(&self) -> usize {
        match self {
            BaseDataValueType::F32(_) => 4,
            BaseDataValueType::Vec2(_) => 8,
            BaseDataValueType::Vec3(_) => 12,
            BaseDataValueType::Vec4(_) => 16,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StructField {
    pub name: String,
    pub data_type: BaseDataValueType,
}
