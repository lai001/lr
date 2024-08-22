use crate::misc::is_dev_mode;
use std::path::{Path, PathBuf};

#[cfg(feature = "editor")]
pub fn get_engine_root_dir() -> PathBuf {
    let path: PathBuf;
    if is_dev_mode() {
        path = Path::new(file!()).join("../../../").to_path_buf();
    } else {
        path = Path::new("../../../").to_path_buf();
    }
    path
}

#[cfg(feature = "editor")]
pub fn get_engine_resource_dir() -> PathBuf {
    get_engine_root_dir().join("Resource")
}

#[cfg(feature = "editor")]
pub fn get_engine_resource(name: &str) -> PathBuf {
    get_engine_resource_dir().join(name)
}

#[cfg(feature = "editor")]
pub fn get_editor_tmp_folder() -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("tmp")
}

#[cfg(feature = "editor")]
pub fn create_editor_tmp_folder() -> PathBuf {
    let path = get_editor_tmp_folder();
    if path.exists() {
        path
    } else {
        std::fs::create_dir_all(path.clone()).unwrap();
        path
    }
}

#[cfg(feature = "editor")]
pub fn get_gpmetis_program_path() -> PathBuf {
    get_engine_root_dir().join("build/windows/x64/release/gpmetis.exe")
}
