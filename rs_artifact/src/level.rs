use crate::{asset::Asset, resource_type::EResourceType};
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

#[typetag::serde(tag = "type")]
pub trait Node: typetag::Serialize + typetag::Deserialize + DynClone {
    fn childs(&self) -> Vec<Box<dyn Node>>;
}
dyn_clone::clone_trait_object!(Node);

#[derive(Serialize, Deserialize, Clone)]
pub struct Node3D {
    pub name: String,
    pub id: uuid::Uuid,
    pub url: url::Url,
    pub mesh_url: Option<url::Url>,
    pub childs: Vec<Box<dyn Node>>,
}

#[typetag::serde]
impl Node for Node3D {
    fn childs(&self) -> Vec<Box<dyn Node>> {
        self.childs.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub id: uuid::Uuid,
    pub url: url::Url,
    pub nodes: Vec<Box<dyn Node>>,
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
    use super::{Node, Node3D};
    use crate::default_url;

    #[test]
    fn test_case() {
        let mut nodes: Vec<Box<dyn Node>> = vec![];
        for x in 0..10 {
            nodes.push(Box::new(Node3D {
                name: x.to_string(),
                id: uuid::Uuid::new_v4(),
                url: default_url().clone(),
                childs: vec![],
                mesh_url: None,
            }));
        }

        let root_node = Node3D {
            name: "root".to_string(),
            id: uuid::Uuid::new_v4(),
            url: default_url().clone(),
            childs: nodes,
            mesh_url: None,
        };

        let data = serde_json::ser::to_string(&root_node).expect("Success.");
        println!("{data}");
    }
}
