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

#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    pub name: String,
    pub nodes: Vec<Rc<RefCell<Node>>>,
}

impl Level {
    pub fn empty_level() -> Self {
        Self {
            name: "Empty".to_string(),
            nodes: vec![],
        }
    }
}
