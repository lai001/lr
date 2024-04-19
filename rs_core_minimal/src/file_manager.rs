use crate::misc::is_run_from_ide;
use std::path::{Path, PathBuf};

#[cfg(feature = "editor")]
pub fn get_engine_root_dir() -> PathBuf {
    let path: PathBuf;
    if is_run_from_ide() {
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
