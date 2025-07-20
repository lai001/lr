use crate::content::content_file_type::EContentFileType;
use rs_artifact::asset::Asset;
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use std::{collections::HashMap, thread::AccessError};

pub struct ContentFileManager {
    content_files: HashMap<url::Url, EContentFileType>,
}

macro_rules! find_by_url {
    (
        $fn_name:ident,
        $variant:ident,
        $type_path:path
    ) => {
        pub fn $fn_name(&self, url: &url::Url) -> Option<SingleThreadMutType<$type_path>> {
            let Some(found) = self.find_by_url(url) else {
                return None;
            };
            match found {
                EContentFileType::$variant(x) => {
                    if x.borrow().get_url() == *url {
                        Some(x.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    };
}

impl ContentFileManager {
    pub fn new() -> Self {
        Self {
            content_files: HashMap::new(),
        }
    }

    pub fn get_unchecked() -> SingleThreadMutType<ContentFileManager> {
        unsafe {
            CONTENT_FILE_MANAGER
                .try_with(|x| x.clone())
                .unwrap_unchecked()
        }
    }

    pub fn get() -> Result<SingleThreadMutType<ContentFileManager>, AccessError> {
        CONTENT_FILE_MANAGER.try_with(|x| x.clone())
    }

    pub fn find_by_url(&self, url: &url::Url) -> Option<EContentFileType> {
        self.content_files.get(url).cloned()
    }

    pub fn add_content(&mut self, content: EContentFileType) -> Option<EContentFileType> {
        self.content_files.insert(content.get_url(), content)
    }

    pub fn files(&self) -> Vec<EContentFileType> {
        self.content_files.values().map(|x| x.clone()).collect()
    }

    pub fn files_map(&self) -> &HashMap<url::Url, EContentFileType> {
        &self.content_files
    }

    find_by_url!(
        find_static_mesh_by_url,
        StaticMesh,
        super::static_mesh::StaticMesh
    );

    find_by_url!(find_material_by_url, Material, super::material::Material);
}

thread_local! {
    static CONTENT_FILE_MANAGER: SingleThreadMutType<ContentFileManager> = SingleThreadMut::new(ContentFileManager::new());
}
