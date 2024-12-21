use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub trait WgslMemoryLayout {
    fn align_of(&self) -> usize;
    fn size_of(&self) -> usize;
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum BaseDataValueType {
    F32(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
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

#[derive(Serialize, Deserialize, Clone)]
pub struct StructField {
    pub name: String,
    pub data_type: BaseDataValueType,
}

#[derive(Clone)]
struct Range {
    field: StructField,
    offset: usize,
}

#[derive(Clone)]
pub struct UniformMap {
    data: Vec<u8>,
    name_range: HashMap<String, Range>,
}

impl UniformMap {
    pub fn new(fields: &[StructField]) -> Self {
        let iter = fields.iter().map(|x| x.name.clone());
        let set = HashSet::<String>::from_iter(iter);
        debug_assert_eq!(set.len(), fields.len());

        let mut size: usize = 0;
        let mut name_range = HashMap::new();
        let mut align = 0;

        for field in fields {
            let field_size = field.data_type.size_of();
            let field_align = field.data_type.align_of();
            align = align.max(field_align);
            let size_padding = rs_foundation::size_padding_of(size, field_align);
            let offset = size + size_padding;
            size = offset + field_size;
            name_range.insert(
                field.name.clone(),
                Range {
                    field: field.clone(),
                    offset,
                },
            );
        }
        let size_padding = rs_foundation::size_padding_of(size, align);
        size = size + size_padding;
        let mut this = Self {
            data: vec![0; size],
            name_range,
        };
        this.transmit();
        this
    }

    pub fn get_field_value_as_f32(&mut self, name: &str) -> Option<f32> {
        let Some(range) = self.name_range.get(name) else {
            return None;
        };
        let BaseDataValueType::F32(_) = range.field.data_type else {
            return None;
        };
        let offset = range.offset;
        let size = range.field.data_type.size_of();
        let block = &mut self.data[offset..offset + size];
        let ptr = unsafe { (block.as_ptr() as *const f32).as_ref().unwrap() };
        Some(*ptr)
    }

    pub fn get_field_value_as_vec2(&mut self, name: &str) -> Option<glam::Vec2> {
        let Some(range) = self.name_range.get(name) else {
            return None;
        };
        let BaseDataValueType::Vec2(_) = range.field.data_type else {
            return None;
        };
        let offset = range.offset;
        let size = range.field.data_type.size_of();
        let block = &mut self.data[offset..offset + size];
        let ptr = unsafe { (block.as_ptr() as *const glam::Vec2).as_ref().unwrap() };
        Some(*ptr)
    }

    pub fn get_field_value_as_vec3(&mut self, name: &str) -> Option<glam::Vec3> {
        let Some(range) = self.name_range.get(name) else {
            return None;
        };
        let BaseDataValueType::Vec3(_) = range.field.data_type else {
            return None;
        };
        let offset = range.offset;
        let size = range.field.data_type.size_of();
        let block = &mut self.data[offset..offset + size];
        let ptr = unsafe { (block.as_ptr() as *const glam::Vec3).as_ref().unwrap() };
        Some(*ptr)
    }

    pub fn get_field_value_as_vec4(&mut self, name: &str) -> Option<glam::Vec4> {
        let Some(range) = self.name_range.get(name) else {
            return None;
        };
        let BaseDataValueType::Vec4(_) = range.field.data_type else {
            return None;
        };
        let offset = range.offset;
        let size = range.field.data_type.size_of();
        let block = &mut self.data[offset..offset + size];
        let ptr = unsafe { (block.as_ptr() as *const glam::Vec4).as_ref().unwrap() };
        Some(*ptr)
    }

    pub fn set_field_f32_value(&mut self, name: &str, value: f32) -> bool {
        let Some(range) = self.name_range.get_mut(name) else {
            return false;
        };
        let BaseDataValueType::F32(field_value) = &mut range.field.data_type else {
            return false;
        };
        *field_value = value;

        let offset = range.offset;
        let size = range.field.data_type.size_of();
        let block = &mut self.data[offset..offset + size];
        let ptr = unsafe { (block.as_mut_ptr() as *mut f32).as_mut().unwrap() };
        *ptr = value;
        true
    }

    pub fn set_field_vec2_value(&mut self, name: &str, value: glam::Vec2) -> bool {
        let Some(range) = self.name_range.get_mut(name) else {
            return false;
        };
        let BaseDataValueType::Vec2(field_value) = &mut range.field.data_type else {
            return false;
        };
        *field_value = value;

        let offset = range.offset;
        let size = range.field.data_type.size_of();
        let block = &mut self.data[offset..offset + size];
        let ptr = unsafe { (block.as_mut_ptr() as *mut glam::Vec2).as_mut().unwrap() };
        *ptr = value;
        true
    }

    pub fn set_field_vec3_value(&mut self, name: &str, value: glam::Vec3) -> bool {
        let Some(range) = self.name_range.get_mut(name) else {
            return false;
        };
        let BaseDataValueType::Vec3(field_value) = &mut range.field.data_type else {
            return false;
        };
        *field_value = value;

        let offset = range.offset;
        let size = range.field.data_type.size_of();
        let block = &mut self.data[offset..offset + size];
        let ptr = unsafe { (block.as_mut_ptr() as *mut glam::Vec3).as_mut().unwrap() };
        *ptr = value;
        true
    }

    pub fn set_field_vec4_value(&mut self, name: &str, value: glam::Vec4) -> bool {
        let Some(range) = self.name_range.get_mut(name) else {
            return false;
        };
        let BaseDataValueType::Vec4(field_value) = &mut range.field.data_type else {
            return false;
        };
        *field_value = value;

        let offset = range.offset;
        let size = range.field.data_type.size_of();
        let block = &mut self.data[offset..offset + size];
        let ptr = unsafe { (block.as_mut_ptr() as *mut glam::Vec4).as_mut().unwrap() };
        *ptr = value;
        true
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    fn transmit(&mut self) {
        for (name, range) in self.name_range.clone() {
            match range.field.data_type {
                BaseDataValueType::F32(value) => {
                    self.set_field_f32_value(&name, value);
                }
                BaseDataValueType::Vec2(value) => {
                    self.set_field_vec2_value(&name, value);
                }
                BaseDataValueType::Vec3(value) => {
                    self.set_field_vec3_value(&name, value);
                }
                BaseDataValueType::Vec4(value) => {
                    self.set_field_vec4_value(&name, value);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{StructField, UniformMap};
    use crate::uniform_map::BaseDataValueType;

    #[test]
    fn test() {
        let fields = vec![
            StructField {
                name: String::from("v1"),
                data_type: BaseDataValueType::F32(0.0),
            },
            StructField {
                name: String::from("v2"),
                data_type: BaseDataValueType::F32(0.0),
            },
            StructField {
                name: String::from("v3"),
                data_type: BaseDataValueType::Vec4(glam::Vec4::ONE),
            },
            StructField {
                name: String::from("v4"),
                data_type: BaseDataValueType::F32(0.0),
            },
        ];
        let mut uniform_map = UniformMap::new(&fields);
        assert_eq!(48, uniform_map.data.len());
        assert!(uniform_map.set_field_f32_value("v2", 100.0));
        assert!(uniform_map.set_field_vec4_value("v3", glam::Vec4::ONE));
        assert_eq!(
            uniform_map.set_field_vec4_value("v4", glam::Vec4::ONE),
            false
        );
        assert_eq!(100.0, uniform_map.get_field_value_as_f32("v2").unwrap());
        assert_eq!(
            glam::Vec4::ONE,
            uniform_map.get_field_value_as_vec4("v3").unwrap()
        );
    }
}
