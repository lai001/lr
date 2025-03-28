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
pub fn get_engine_output_target_dir() -> PathBuf {
    get_engine_root_dir().join("build/target")
}

#[cfg(feature = "editor")]
pub fn get_engine_generated_dir() -> PathBuf {
    get_engine_root_dir().join("build/target/generated")
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

thread_local! {
    pub static GLOBAL_CURRENT_PROJECT_DIR: std::cell::RefCell<PathBuf>  = std::cell::RefCell::new(PathBuf::new()) ;
}

#[cfg(feature = "editor")]
pub fn get_current_project_dir() -> PathBuf {
    GLOBAL_CURRENT_PROJECT_DIR.with(|x| x.borrow().clone())
}

#[cfg(feature = "editor")]
pub fn set_current_project_dir(path: &Path) {
    GLOBAL_CURRENT_PROJECT_DIR.with_borrow_mut(|x| {
        *x = path.to_path_buf();
    });
}

pub fn get_current_exe_dir() -> crate::error::Result<PathBuf> {
    let current_exe = std::env::current_exe().map_err(|err| crate::error::Error::IO(err))?;
    let parent = current_exe
        .parent()
        .ok_or(crate::error::Error::IO(std::io::ErrorKind::NotFound.into()))?;
    Ok(parent.to_path_buf())
}
