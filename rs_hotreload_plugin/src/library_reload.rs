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

    pub fn load_symbol<Signature>(
        &self,
        symbol_name: &str,
    ) -> Result<libloading::Symbol<Signature>> {
        match self.libraries.last() {
            Some(ref x) => unsafe {
                let symbol = match x.get(symbol_name.as_bytes()) {
                    Ok(symbol) => symbol,
                    Err(err) => {
                        return Err(crate::error::Error::Libloading(err, None));
                    }
                };
                Ok(symbol)
            },
            None => {
                return Err(crate::error::Error::IO(
                    std::io::ErrorKind::NotFound.into(),
                    None,
                ));
            }
        }
    }

    pub(crate) fn reload(&mut self) -> Result<()> {
        let new_index = self.index + 1;

        let target_file_path = match Self::copy_library(self.index, &self.folder, &self.lib_name) {
            Ok(target_file_path) => target_file_path,
            Err(err) => {
                return Err(err);
            }
        };

        let library = match Self::load_library(&target_file_path) {
            Ok(library) => library,
            Err(err) => {
                return Err(crate::error::Error::Libloading(err, None));
            }
        };

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
            return format!("{}{}_{}.{}", prefix, lib_name, index, extension);
        } else {
            return format!("{}{}.{}", prefix, lib_name, extension);
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
        match std::fs::copy(original_file_path.clone(), target_file_path.clone()) {
            Ok(_) => Ok(target_file_path.to_string_lossy().to_string()),
            Err(err) => Err(crate::error::Error::IO(
                err,
                Some(format!(
                    "Can not copy {:?} to {:?}",
                    original_file_path, target_file_path
                )),
            )),
        }
    }

    fn sear_max_number(folder: &str, lib_name: &str) -> i32 {
        let reg = regex::Regex::new(r"_\d*").unwrap();
        let number_reg = regex::Regex::new(r"[0-9]*$").unwrap();

        let mut numbers: Vec<i32> = vec![];
        for entry in WalkDir::new(folder).max_depth(1) {
            if let Ok(entry) = entry {
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
                        .starts_with(&(Self::get_lib_name_prexif().to_string() + lib_name + "_"))
                {
                    let captures = reg.captures(&file_stem).unwrap();
                    let number = number_reg.captures(&captures[0]).unwrap();

                    numbers.push(number.get(0).unwrap().as_str().parse::<i32>().unwrap());
                }
            }
        }
        numbers.sort();
        let max = numbers.last().unwrap_or(&0);
        *max
    }

    pub fn get_original_lib_file_path(&self) -> PathBuf {
        let path = Path::new(&self.folder).join(&Self::get_full_lib_filename(&self.lib_name, None));
        path
    }

    pub fn clean_cache(&self) {
        for entry in WalkDir::new(self.folder.clone()).max_depth(1) {
            if let Ok(entry) = entry {
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
                    && file_stem.starts_with(
                        &(Self::get_lib_name_prexif().to_string() + &self.lib_name + "_"),
                    )
                {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::LibraryReload;
    use std::{path::Path, process::Command};

    #[test]
    pub fn test_case() {
        let binding = std::env::current_exe().unwrap();
        let work_dir = std::path::Path::new(&binding).parent().unwrap();
        std::env::set_current_dir(&work_dir).unwrap();

        let path = work_dir.join("test.dll");
        std::fs::write(path, "").unwrap();
        for i in 0..20 {
            let path = work_dir.join(format!("test_{}.dll", i));
            std::fs::write(path, "").unwrap();
        }

        let max_number = LibraryReload::sear_max_number(work_dir.to_str().unwrap(), "test");
        assert_eq!(max_number, 19);
    }

    pub fn compile_test_lib(work_dir: &Path, source_code: &str) -> String {
        std::fs::write(work_dir.join("test.rs"), source_code).unwrap();
        let mut compile_command = Command::new("rustc");
        compile_command.args([
            "--crate-name",
            "test",
            "--edition=2021",
            work_dir.join("test.rs").to_str().unwrap(),
            "--crate-type",
            "dylib",
        ]);
        let child = compile_command.spawn().unwrap();
        let _ = child.wait_with_output().unwrap();
        return work_dir.join("test.dll").to_string_lossy().to_string();
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
        );
        hot_reload.reload().unwrap();
        let symbol = hot_reload
            .load_symbol::<fn(usize, usize) -> usize>("add")
            .unwrap();
        let result = symbol(1, 1);
        assert_eq!(result, 3);
    }
}
