use crate::{asset::Asset, property_value_type::EPropertyValueType, resource_type::EResourceType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Node3D {
    pub name: String,
    pub id: uuid::Uuid,
    pub url: Option<url::Url>,
    pub mesh_url: Option<url::Url>,
    pub values: HashMap<String, EPropertyValueType>,
    pub childs: Vec<ENodeType>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ENodeType {
    Node3D(Node3D),
}

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub id: uuid::Uuid,
    pub url: url::Url,
    pub nodes: Vec<ENodeType>,
}

impl Asset for Level {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Level
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::Node3D;
    use crate::{default_url, level::ENodeType};

    #[test]
    fn test_case() {
        let mut nodes: Vec<ENodeType> = vec![];
        for x in 0..10 {
            nodes.push(ENodeType::Node3D(Node3D {
                name: x.to_string(),
                id: uuid::Uuid::new_v4(),
                url: Some(default_url().clone()),
                childs: vec![],
                mesh_url: None,
                values: HashMap::new(),
            }));
        }

        let root_node = Node3D {
            name: "root".to_string(),
            id: uuid::Uuid::new_v4(),
            url: Some(default_url().clone()),
            childs: nodes,
            mesh_url: None,
            values: HashMap::new(),
        };

        let data = serde_json::ser::to_string(&root_node).expect("Success.");
        println!("{data}");
    }
}
