use crate::static_mesh::StaticMesh;
use crate::texture::TextureFile;
use crate::{node_animation::NodeAnimation, skeleton::Skeleton, skeleton_mesh::SkeletonMesh};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EContentFileType {
    StaticMesh(Rc<RefCell<StaticMesh>>),
    SkeletonMesh(Rc<RefCell<SkeletonMesh>>),
    NodeAnimation(Rc<RefCell<NodeAnimation>>),
    Skeleton(Rc<RefCell<Skeleton>>),
    Texture(Rc<RefCell<TextureFile>>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentFolder {
    pub name: String,
    pub parent_folder: Option<Rc<RefCell<ContentFolder>>>,
    pub folders: Vec<Rc<RefCell<ContentFolder>>>,
    pub files: Vec<EContentFileType>,
}

impl ContentFolder {
    pub fn new<S: AsRef<str>>(name: S, parent_folder: Option<Rc<RefCell<ContentFolder>>>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            files: vec![],
            folders: vec![],
            parent_folder,
        }
    }

    pub fn get_url(&self) -> url::Url {
        let mut components: Vec<String> = vec![];
        components.push(self.name.clone());
        let mut parent_folder = self.parent_folder.clone();
        while let Some(folder) = parent_folder {
            components.push(folder.borrow().name.clone());
            parent_folder = folder.borrow().parent_folder.clone();
        }
        components.reverse();
        let mut path: String = "".to_string();
        for component in &components {
            path = format!("/{}", component);
        }
        let url = url::Url::parse(&format!("content:/{}", path)).unwrap();
        url
    }
}

impl Default for ContentFolder {
    fn default() -> Self {
        Self {
            name: "Content".to_string(),
            files: vec![],
            folders: vec![],
            parent_folder: None,
        }
    }
}
