use crate::project::ProjectDescription;
use std::path::Path;
use std::sync::{Arc, Mutex};

struct STFileManager {
    project_description: ProjectDescription,
}

lazy_static! {
    static ref GLOBAL_FILEMANAGER: Arc<FileManager> = Arc::new(FileManager::new());
}

impl STFileManager {
    fn new() -> STFileManager {
        STFileManager {
            project_description: ProjectDescription::current(),
        }
    }

    fn get_resource_dir_path(&self) -> String {
        self.project_description.get_paths().resource_dir.clone()
    }

    fn get_resource_path(&self, resource_name: &str) -> String {
        Path::join(
            Path::new(&self.project_description.get_paths().resource_dir),
            resource_name,
        )
        .to_str()
        .unwrap()
        .to_string()
    }

    fn get_shader_dir_path(&self) -> String {
        self.project_description.get_paths().shader_dir.clone()
    }

    fn get_intermediate_dir_path(&self) -> String {
        self.project_description
            .get_paths()
            .intermediate_dir
            .clone()
    }

    fn get_project_description(&self) -> ProjectDescription {
        self.project_description.clone()
    }

    fn get_user_script_path(&self) -> String {
        self.project_description.get_user_script().path.clone()
    }
}

pub struct FileManager {
    inner: Mutex<STFileManager>,
}

impl FileManager {
    pub fn new() -> FileManager {
        FileManager {
            inner: Mutex::new(STFileManager::new()),
        }
    }

    pub fn default() -> Arc<FileManager> {
        GLOBAL_FILEMANAGER.clone()
    }

    pub fn get_resource_dir_path(&self) -> String {
        self.inner.lock().unwrap().get_resource_dir_path()
    }

    pub fn get_resource_path(&self, resource_name: &str) -> String {
        self.inner.lock().unwrap().get_resource_path(resource_name)
    }

    pub fn get_shader_dir_path(&self) -> String {
        self.inner.lock().unwrap().get_shader_dir_path()
    }

    pub fn get_intermediate_dir_path(&self) -> String {
        self.inner.lock().unwrap().get_intermediate_dir_path()
    }

    pub fn get_project_description(&self) -> ProjectDescription {
        self.inner.lock().unwrap().get_project_description()
    }

    pub fn get_user_script_path(&self) -> String {
        self.inner.lock().unwrap().get_user_script_path()
    }
}
