use path_slash::PathBufExt;
use std::path::{Path, PathBuf};

pub trait CanonicalizeSlashExt {
    fn canonicalize_slash(&self) -> std::io::Result<PathBuf>;
}

impl CanonicalizeSlashExt for Path {
    fn canonicalize_slash(&self) -> std::io::Result<PathBuf> {
        match dunce::canonicalize(self) {
            Ok(path) => Ok(Path::new(&path.to_string_lossy().to_string()).to_path_buf()),
            Err(err) => Err(err),
        }
    }
}

impl CanonicalizeSlashExt for PathBuf {
    fn canonicalize_slash(&self) -> std::io::Result<PathBuf> {
        match dunce::canonicalize(self) {
            Ok(path) => Ok(Path::new(&path.to_slash_lossy().to_string()).to_path_buf()),
            Err(err) => Err(err),
        }
    }
}

impl CanonicalizeSlashExt for &PathBuf {
    fn canonicalize_slash(&self) -> std::io::Result<PathBuf> {
        match dunce::canonicalize(self) {
            Ok(path) => Ok(Path::new(&path.to_slash_lossy().to_string()).to_path_buf()),
            Err(err) => Err(err),
        }
    }
}
