use rs_engine::content::{
    content_file_type::EContentFileType,
    material_paramenters_collection::MaterialParamentersCollection,
};
use rs_foundation::new::SingleThreadMutType;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Serialize, Deserialize, Clone)]
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

    pub fn collect_material_parameters_collections(
        &self,
        is_recursion: bool,
    ) -> Vec<SingleThreadMutType<MaterialParamentersCollection>> {
        let mut material_parameters_collections = vec![];
        for file in self.files.clone() {
            match file {
                EContentFileType::MaterialParamentersCollection(material_parameters_collection) => {
                    material_parameters_collections.push(material_parameters_collection);
                }
                _ => {}
            }
        }
        if is_recursion {
            for folder in self.folders.clone() {
                let folder = folder.borrow();
                let mut child_folder_files =
                    folder.collect_material_parameters_collections(is_recursion);
                material_parameters_collections.append(&mut child_folder_files);
            }
        }
        material_parameters_collections
    }

    pub fn files_to_map(&self, is_recursion: bool) -> HashMap<url::Url, EContentFileType> {
        let mut map = HashMap::new();
        for file in self.files.clone() {
            map.insert(file.get_url(), file);
        }
        if is_recursion {
            for folder in self.folders.clone() {
                let folder = folder.borrow();
                let child_map = folder.files_to_map(is_recursion);
                for (key, value) in child_map {
                    map.insert(key, value);
                }
            }
        }
        map
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
