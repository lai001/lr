use crate::{actor::Actor, property};
use rs_artifact::property_value_type::EPropertyValueType;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct MeshReference {
    pub file_path: PathBuf,
    pub referenced_mesh_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: uuid::Uuid,
    pub name: String,
    pub mesh_reference: Option<MeshReference>,
    pub values: HashMap<String, EPropertyValueType>,
    pub childs: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn get_model_matrix(&self) -> Option<glam::Mat4> {
        let mut scale_matrix: Option<glam::Mat4> = None;
        let mut translation_matrix: Option<glam::Mat4> = None;
        let mut rotation_matrix: Option<glam::Mat4> = None;

        if let Some(scale) = self.values.get(property::name::SCALE) {
            if let EPropertyValueType::Vec3(scale) = scale {
                scale_matrix = Some(glam::Mat4::from_scale(*scale));
            }
        }
        if let Some(rotation) = self.values.get(property::name::ROTATION) {
            if let EPropertyValueType::Quat(rotation) = rotation {
                rotation_matrix = Some(glam::Mat4::from_quat(*rotation));
            }
        }
        if let Some(translation) = self.values.get(property::name::TRANSLATION) {
            if let EPropertyValueType::Vec3(translation) = translation {
                translation_matrix = Some(glam::Mat4::from_translation(*translation));
            }
        }
        if let (Some(s), Some(t), Some(r)) = (scale_matrix, translation_matrix, rotation_matrix) {
            return Some(t * r * s);
        } else {
            return None;
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub nodes: Vec<Rc<RefCell<Node>>>,
    pub actors: Vec<Rc<RefCell<Actor>>>,
}

impl Level {
    pub fn empty_level() -> Self {
        Self {
            name: "Empty".to_string(),
            nodes: vec![],
            actors: vec![],
        }
    }
}

pub fn default_node3d_properties() -> HashMap<String, EPropertyValueType> {
    HashMap::from([
        (
            property::name::TEXTURE.to_string(),
            EPropertyValueType::Texture(None),
        ),
        (
            property::name::SCALE.to_string(),
            EPropertyValueType::Vec3(glam::Vec3::ONE),
        ),
        (
            property::name::TRANSLATION.to_string(),
            EPropertyValueType::Vec3(glam::Vec3::ZERO),
        ),
        (
            property::name::ROTATION.to_string(),
            EPropertyValueType::Quat(glam::Quat::IDENTITY),
        ),
    ])
}
