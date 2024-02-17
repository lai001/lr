use crate::path_ext::CanonicalizeSlashExt;
use std::path::{Path, PathBuf};

pub fn get_engine_root_dir() -> PathBuf {
    Path::new(file!())
        .join("../../../")
        .canonicalize_slash()
        .unwrap()
}

pub fn get_engine_resource_dir() -> PathBuf {
    get_engine_root_dir().join("Resource")
}

pub fn get_engine_resource(name: &str) -> PathBuf {
    get_engine_resource_dir().join(name)
}
