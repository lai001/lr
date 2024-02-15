use crate::path_ext::CanonicalizeSlashExt;
use std::path::{Path, PathBuf};

pub fn get_engine_root_dir() -> PathBuf {
    Path::new(file!())
        .join("../../../")
        .canonicalize_slash()
        .unwrap()
}
