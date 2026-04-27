use rs_core_minimal::need_copy::{is_need_copy, CompareMode};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    time::SystemTime,
};

pub fn copy_to_output(src: &Path) {
    let cargo_pkg_name = build_rs::input::cargo_pkg_name();
    let out_dir = build_rs::input::out_dir();
    let exe_dir = out_dir.ancestors().nth(3).unwrap().to_path_buf();
    let dst = exe_dir.join(src.file_name().unwrap());
    build_rs::output::rerun_if_changed(&src);
    build_rs::output::rerun_if_changed(&dst);
    if is_need_copy(src, dst.as_ref(), CompareMode::SIZE | CompareMode::MTIME).unwrap() {
        build_print::info!(
            "[{}] Copy {} to {}",
            cargo_pkg_name,
            src.display(),
            dst.display()
        );
        fs::copy(src, &dst).unwrap();
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FileMetadata {
    last_write_time: SystemTime,
    file_size: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct FilesManifest {
    pub files: std::collections::HashMap<PathBuf, FileMetadata>,
}

impl FilesManifest {
    pub fn from<P: AsRef<Path>>(file_paths: std::collections::HashSet<P>) -> Option<FilesManifest> {
        let mut files = std::collections::HashMap::new();
        files.reserve(file_paths.len());
        for file_path in file_paths {
            let file_path = file_path.as_ref();
            let metadata = file_path.metadata().ok()?;
            let modified = metadata.modified().ok()?;
            let file_size = metadata.file_size();
            files.insert(
                file_path.to_path_buf(),
                FileMetadata {
                    last_write_time: modified,
                    file_size,
                },
            );
        }
        Some(FilesManifest { files })
    }

    pub fn load_from<P: AsRef<Path>>(file_path: P) -> Option<FilesManifest> {
        let file = fs::File::open(file_path).ok()?;
        let reader = std::io::BufReader::new(file);
        let manifest = serde_json::from_reader(reader).ok()?;
        Some(manifest)
    }

    pub fn compare<P: AsRef<Path>>(
        file_paths: std::collections::HashSet<P>,
        file_path: P,
    ) -> Option<bool> {
        let current = FilesManifest::from(file_paths)?;
        let cache = FilesManifest::load_from(file_path)?;
        Some(current == cache)
    }

    pub fn save_to<P: AsRef<Path>>(&self, file_path: P) -> bool {
        let Ok(file) = fs::File::create(file_path) else {
            return false;
        };
        serde_json::to_writer(file, self).is_ok()
    }
}

// https://github.com/rust-lang/cargo/issues/15716
pub struct RerunIfChangedContext {
    file_paths: std::collections::HashSet<PathBuf>,
    cache_path: PathBuf,
}

impl RerunIfChangedContext {
    pub fn new(
        file_paths: std::collections::HashSet<PathBuf>,
        cache_path: PathBuf,
    ) -> RerunIfChangedContext {
        RerunIfChangedContext {
            cache_path,
            file_paths,
        }
    }

    pub fn insert(&mut self, path: PathBuf) {
        self.file_paths.insert(path);
    }

    pub fn rerun_if_changed(self) -> bool {
        let Some(current) = FilesManifest::from(self.file_paths) else {
            return true;
        };
        let Some(cache) = FilesManifest::load_from(&self.cache_path) else {
            return true;
        };

        for (cache_path, cache_file_metadata) in &cache.files {
            if !cache_path.exists() {
                return true;
            }

            let Ok(metadata) = cache_path.metadata() else {
                return true;
            };
            if metadata.file_size() != cache_file_metadata.file_size {
                return true;
            }
            if metadata.modified().ok() != Some(cache_file_metadata.last_write_time) {
                return true;
            }
        }

        current.save_to(&self.cache_path);
        current == cache
    }
}

#[cfg(test)]
pub mod test {
    use crate::{FilesManifest, RerunIfChangedContext};
    use std::collections::HashSet;

    #[test]
    fn files_manifest_test() {
        let manifest_dir = build_rs::input::cargo_manifest_dir();
        let pkg_name = manifest_dir.file_name().unwrap();

        let out_dir = rs_core_minimal::file_manager::get_engine_build_tmp_dir().join(pkg_name);
        let _ = std::fs::create_dir_all(&out_dir).unwrap();
        let out_dir = out_dir.canonicalize().unwrap();

        let file1 = out_dir.join("files_manifest_test_file1.txt");
        let file2 = out_dir.join("files_manifest_test_file2.txt");
        let _ = std::fs::remove_file(&file1);
        let _ = std::fs::remove_file(&file2);

        assert!(std::fs::write(&file1, "contents").is_ok());
        assert!(std::fs::write(&file2, "contents").is_ok());

        let Some(cache) = FilesManifest::from(HashSet::from([&file1, &file2])) else {
            panic!()
        };

        let manifest_file_path = out_dir.join("files_manifest_test.manifest.json");
        assert!(cache.save_to(&manifest_file_path));

        assert!(std::fs::write(&file1, "new contents").is_ok());
        assert!(std::fs::write(&file2, "new contents").is_ok());

        assert_eq!(
            Some(false),
            FilesManifest::compare(HashSet::from([&file1, &file2]), &manifest_file_path,)
        );
    }

    #[test]
    fn rerun_if_changed_context_test() {
        let manifest_dir = build_rs::input::cargo_manifest_dir();
        let pkg_name = manifest_dir.file_name().unwrap();

        let out_dir = rs_core_minimal::file_manager::get_engine_build_tmp_dir().join(pkg_name);
        let _ = std::fs::create_dir_all(&out_dir).unwrap();
        let out_dir = out_dir.canonicalize().unwrap();

        let file1 = out_dir.join("rerun_if_changed_context_test_file1.txt");
        let file2 = out_dir.join("rerun_if_changed_context_test_file2.txt");
        let _ = std::fs::remove_file(&file1);
        let _ = std::fs::remove_file(&file2);

        assert!(std::fs::write(&file1, "contents").is_ok());
        assert!(std::fs::write(&file2, "contents").is_ok());

        let manifest_file_path = out_dir.join("rerun_if_changed_context_test.manifest.json");

        assert!(std::fs::write(&file1, "new contents").is_ok());
        assert!(std::fs::write(&file2, "new contents").is_ok());

        let ctx = RerunIfChangedContext::new(
            HashSet::from([file1.clone(), file2.clone()]),
            manifest_file_path,
        );
        assert!(ctx.rerun_if_changed());
    }

    #[test]
    fn rerun_if_changed_context_test1() {
        let manifest_dir = build_rs::input::cargo_manifest_dir();
        let pkg_name = manifest_dir.file_name().unwrap();

        let out_dir = rs_core_minimal::file_manager::get_engine_build_tmp_dir().join(pkg_name);
        let _ = std::fs::create_dir_all(&out_dir).unwrap();
        let out_dir = out_dir.canonicalize().unwrap();

        let file1 = out_dir.join("rerun_if_changed_context_test1_file1.txt");
        let file2 = out_dir.join("rerun_if_changed_context_test1_file2.txt");
        let _ = std::fs::remove_file(&file1);
        let _ = std::fs::remove_file(&file2);

        assert!(std::fs::write(&file1, "contents").is_ok());
        assert!(std::fs::write(&file2, "contents").is_ok());

        let Some(cache) = FilesManifest::from(HashSet::from([&file1, &file2])) else {
            panic!()
        };

        let manifest_file_path = out_dir.join("rerun_if_changed_context_test1.manifest.json");
        assert!(cache.save_to(&manifest_file_path));

        assert!(std::fs::write(&file1, "new contents").is_ok());
        assert!(std::fs::write(&file2, "new contents").is_ok());

        let ctx = RerunIfChangedContext::new(
            HashSet::from([file1.clone(), file2.clone()]),
            manifest_file_path,
        );
        assert!(ctx.rerun_if_changed());
    }
}
