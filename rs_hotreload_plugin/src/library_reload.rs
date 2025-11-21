use crate::error::Result;
use libloading::Library;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct LibraryReload {
    folder: PathBuf,
    lib_name: String,
    index: i32,
    libraries: Vec<Library>,
}

impl LibraryReload {
    pub(crate) fn new(folder: &Path, lib_name: &str) -> LibraryReload {
        let hot_reload_lib = LibraryReload {
            folder: folder.to_path_buf(),
            lib_name: lib_name.to_string(),
            index: 0,
            libraries: Vec::new(),
        };
        hot_reload_lib
    }

    pub fn is_loaded(&self) -> bool {
        self.libraries.is_empty() == false
    }

    pub fn load_symbol<'a, Signature>(
        &'a self,
        symbol_name: &str,
    ) -> Result<libloading::Symbol<'a, Signature>> {
        let library = self.libraries.last().ok_or(crate::error::Error::IO(
            std::io::ErrorKind::NotFound.into(),
            None,
        ))?;
        let symbol = unsafe { library.get(symbol_name.as_bytes()) }
            .map_err(|err| crate::error::Error::Libloading(err, None))?;
        Ok(symbol)
    }

    pub(crate) fn reload(&mut self) -> Result<()> {
        let new_index = self.index + 1;
        let target_file_path = Self::copy_library(self.index, &self.folder, &self.lib_name)?;
        let library = Self::load_library(&target_file_path)
            .map_err(|err| crate::error::Error::Libloading(err, None))?;
        self.libraries.push(library);
        self.index = new_index;
        Ok(())
    }

    pub fn get_lib_name_prexif() -> &'static str {
        #[cfg(not(target_os = "windows"))]
        let prefix = "lib";
        #[cfg(target_os = "windows")]
        let prefix = "";
        prefix
    }

    pub fn get_lib_name_extension() -> &'static str {
        #[cfg(target_os = "macos")]
        let extension = "dylib";
        #[cfg(target_os = "linux")]
        let extension = "so";
        #[cfg(target_os = "windows")]
        let extension = "dll";
        extension
    }

    pub fn get_full_lib_filename(lib_name: &str, index: Option<i32>) -> String {
        let prefix = Self::get_lib_name_prexif();
        let extension = Self::get_lib_name_extension();
        if let Some(index) = index {
            format!("{}{}_{}.{}", prefix, lib_name, index, extension)
        } else {
            format!("{}{}.{}", prefix, lib_name, extension)
        }
    }

    fn load_library(path: &str) -> std::result::Result<libloading::Library, libloading::Error> {
        log::trace!("Load {}.", path);
        unsafe { libloading::Library::new(path) }
    }

    fn copy_library(index: i32, folder: &Path, lib_name: &str) -> Result<String> {
        let original_filename = Self::get_full_lib_filename(lib_name, None);
        let original_file_path = std::path::Path::new(folder).join(original_filename);
        let target_filename = Self::get_full_lib_filename(lib_name, Some(index));
        let target_file_path = std::path::Path::new(folder).join(target_filename);
        std::fs::copy(original_file_path.clone(), target_file_path.clone()).map_err(|err| {
            let msg = format!("Can not copy {original_file_path:?} to {target_file_path:?}");
            crate::error::Error::IO(err, Some(msg))
        })?;
        Ok(target_file_path.to_string_lossy().to_string())
    }

    pub fn get_original_lib_file_path(&self) -> PathBuf {
        let path = Path::new(&self.folder).join(&Self::get_full_lib_filename(&self.lib_name, None));
        path
    }

    pub fn clean_cache(&self) {
        for entry in WalkDir::new(self.folder.clone()).max_depth(1) {
            let Ok(entry) = entry else {
                continue;
            };

            let path = entry.path();
            let extension = path
                .extension()
                .unwrap_or(Default::default())
                .to_string_lossy()
                .to_string();
            let file_stem = path
                .file_stem()
                .unwrap_or(Default::default())
                .to_string_lossy()
                .to_string();
            if extension == Self::get_lib_name_extension()
                && file_stem
                    .starts_with(&(Self::get_lib_name_prexif().to_string() + &self.lib_name + "_"))
            {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    pub fn clear(&mut self) {
        self.libraries.clear();
    }
}

#[cfg(test)]
pub mod test {
    use super::LibraryReload;
    use std::{path::Path, process::Command};

    pub fn compile_test_lib(work_dir: &Path, source_code: &str, output_name: &str) -> String {
        std::fs::write(work_dir.join("test.rs"), source_code).unwrap();
        let mut compile_command = Command::new("rustc");
        compile_command.args([
            "--crate-name",
            output_name,
            "--edition=2021",
            work_dir.join("test.rs").to_str().unwrap(),
            "--crate-type",
            "dylib",
        ]);
        let child = compile_command.spawn().unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        return work_dir
            .join(format!("{output_name}.dll"))
            .to_string_lossy()
            .to_string();
    }

    #[test]
    fn test_case_1() {
        let binding = std::env::current_exe().unwrap();
        let work_dir = std::path::Path::new(&binding).parent().unwrap();
        std::env::set_current_dir(&work_dir).unwrap();

        compile_test_lib(
            work_dir,
            r"#[no_mangle]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}",
            "test",
        );

        let mut hot_reload = LibraryReload::new(work_dir, "test");
        assert_eq!(hot_reload.is_loaded(), false);
        hot_reload.reload().unwrap();
        let symbol = hot_reload
            .load_symbol::<fn(usize, usize) -> usize>("add")
            .unwrap();
        let result = symbol(1, 1);
        assert_eq!(result, 2);

        compile_test_lib(
            work_dir,
            r"#[no_mangle]
        pub fn add(left: usize, right: usize) -> usize {
            left + right + 1
        }",
            "test",
        );
        hot_reload.reload().unwrap();
        let symbol = hot_reload
            .load_symbol::<fn(usize, usize) -> usize>("add")
            .unwrap();
        let result = symbol(1, 1);
        assert_eq!(result, 3);
    }
}
