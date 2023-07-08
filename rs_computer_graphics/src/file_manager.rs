use crate::project::ProjectDescription;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct FileManager {
    project_description: Arc<Mutex<ProjectDescription>>,
}

lazy_static! {
    static ref GLOBAL_FILEMANAGER: Arc<Mutex<FileManager>> =
        Arc::new(Mutex::new(FileManager::new()));
}

impl FileManager {
    pub fn new() -> FileManager {
        FileManager {
            project_description: ProjectDescription::default(),
        }
    }

    pub fn default() -> Arc<Mutex<FileManager>> {
        GLOBAL_FILEMANAGER.clone()
    }

    pub fn get_resource_dir_path(&self) -> String {
        self.project_description
            .lock()
            .unwrap()
            .get_paths()
            .resource_dir
            .clone()
    }

    pub fn get_resource_path(&self, resource_name: &str) -> String {
        Path::join(
            Path::new(
                &self
                    .project_description
                    .lock()
                    .unwrap()
                    .get_paths()
                    .resource_dir,
            ),
            resource_name,
        )
        .to_str()
        .unwrap()
        .to_string()
    }

    pub fn get_shader_dir_path(&self) -> String {
        self.project_description
            .lock()
            .unwrap()
            .get_paths()
            .shader_dir
            .clone()
    }

    pub fn get_intermediate_dir_path(&self) -> String {
        self.project_description
            .lock()
            .unwrap()
            .get_paths()
            .intermediate_dir
            .clone()
    }
}
