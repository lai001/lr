use crate::path_ext::CanonicalizeSlashExt;
use std::path::{Path, PathBuf};

#[cfg(feature = "editor")]
pub fn get_engine_root_dir() -> PathBuf {
    Path::new(file!())
        .join("../../../")
        .canonicalize_slash()
        .unwrap()
}

#[cfg(feature = "editor")]
pub fn get_engine_resource_dir() -> PathBuf {
    get_engine_root_dir().join("Resource")
}

#[cfg(feature = "editor")]
pub fn get_engine_resource(name: &str) -> PathBuf {
    get_engine_resource_dir().join(name)
}
