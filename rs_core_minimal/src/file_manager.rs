use crate::path_ext::CanonicalizeSlashExt;
use std::path::{Path, PathBuf};

#[cfg(feature = "editor")]
pub fn is_run_from_ide() -> bool {
    let vars = std::env::vars().filter(|x| x.0 == "CARGO_MANIFEST_DIR".to_string());
    vars.count() != 0
}

#[cfg(feature = "editor")]
pub fn get_engine_root_dir() -> PathBuf {
    if is_run_from_ide() {
        Path::new(file!())
            .join("../../../")
            .canonicalize_slash()
            .unwrap()
    } else {
        Path::new("../../../").canonicalize_slash().unwrap()
    }
}

#[cfg(feature = "editor")]
pub fn get_engine_resource_dir() -> PathBuf {
    get_engine_root_dir().join("Resource")
}

#[cfg(feature = "editor")]
pub fn get_engine_resource(name: &str) -> PathBuf {
    get_engine_resource_dir().join(name)
}
