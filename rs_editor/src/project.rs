use crate::{error::Result, level::Level};
use path_slash::PathBufExt;
use rs_artifact::EEndianType;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    io::Write,
    path::{Path, PathBuf},
    rc::Rc,
};

pub const PROJECT_FILE_EXTENSION: &str = "rsproject";
pub const ASSET_FOLDER_NAME: &str = "asset";
pub const BUILD_FOLDER_NAME: &str = "build";
pub const SHADER_FOLDER_NAME: &str = "shader";
pub const SRC_FOLDER_NAME: &str = "src";
pub const VERSION_STR: &str = "0.0.1";

#[derive(Serialize, Deserialize, Debug)]
pub struct VirtualTextureSetting {
    pub tile_size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub version_str: String,
    pub project_name: String,
    pub virtual_texture_setting: VirtualTextureSetting,
    pub endian_type: EEndianType,
    pub level: Rc<RefCell<Level>>,
    pub texture_folder: crate::texture::TextureFolder,
}

impl Project {
    pub fn create_empty_project(
        project_parent_folder: &Path,
        project_name: &str,
    ) -> Result<PathBuf> {
        if let Err(err) = Self::create_empty_project_folders(project_parent_folder, project_name) {
            return Err(crate::error::Error::IO(err, None));
        }
        if Self::create_empty_project_file_to_disk(&project_parent_folder, project_name) == false {
            return Err(crate::error::Error::CreateProjectFailed);
        }
        if Self::create_cargo_toml_file(project_parent_folder, project_name) == false {
            return Err(crate::error::Error::CreateProjectFailed);
        }
        if Self::create_lib_file(project_parent_folder, project_name) == false {
            return Err(crate::error::Error::CreateProjectFailed);
        }
        let project_folder = project_parent_folder.join(project_name);
        let project_file_path =
            project_folder.join(format!("{}.{}", project_name, PROJECT_FILE_EXTENSION));
        Ok(project_file_path)
    }

    fn root_texture_folder() -> crate::texture::TextureFolder {
        crate::texture::TextureFolder {
            name: String::from("Textures"),
            url: url::Url::parse("texture://Textures").unwrap(),
            texture_files: Vec::new(),
            texture_folders: Vec::new(),
        }
    }

    fn create_empty_project_file_to_disk(project_parent_folder: &Path, project_name: &str) -> bool {
        let project_folder = project_parent_folder.join(project_name);
        let project_file_path =
            project_folder.join(format!("{}.{}", project_name, PROJECT_FILE_EXTENSION));
        if project_file_path.exists() {
            return false;
        }

        let empty_project = Project {
            version_str: VERSION_STR.to_string(),
            project_name: project_name.to_string(),
            level: Rc::new(RefCell::new(Level::empty_level())),
            texture_folder: Self::root_texture_folder(),
            endian_type: EEndianType::Little,
            virtual_texture_setting: VirtualTextureSetting { tile_size: 256 },
        };

        let Ok(json_str) = serde_json::ser::to_string_pretty(&empty_project) else {
            return false;
        };

        let Ok(mut file) = std::fs::File::create(project_file_path) else {
            return false;
        };
        match file.write_fmt(format_args!("{}", json_str)) {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    fn create_empty_project_folders(
        project_parent_folder: &Path,
        project_name: &str,
    ) -> std::io::Result<()> {
        let project_folder = project_parent_folder.join(project_name);
        std::fs::create_dir(project_folder.clone())?;
        std::fs::create_dir(project_folder.join(SRC_FOLDER_NAME))?;
        std::fs::create_dir(project_folder.join(ASSET_FOLDER_NAME))?;
        std::fs::create_dir(project_folder.join(SHADER_FOLDER_NAME))?;
        std::fs::create_dir(project_folder.join(BUILD_FOLDER_NAME))?;
        Ok(())
    }

    fn create_cargo_toml_file(project_parent_folder: &Path, project_name: &str) -> bool {
        let Ok(current_dir) = std::env::current_dir() else {
            return false;
        };
        let Ok(engien_dir) = current_dir.join("../../../").canonicalize() else {
            return false;
        };
        let Ok(engien_dir) = dunce::canonicalize(&engien_dir) else {
            return false;
        };
        let project_folder = project_parent_folder.join(project_name);
        let toml_file_path = project_folder.join("Cargo.toml");
        let content = fill_cargo_toml_template(project_name, &engien_dir.to_slash().unwrap());
        let Ok(mut file) = std::fs::File::create(toml_file_path) else {
            return false;
        };
        match file.write_fmt(format_args!("{}", content)) {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    fn create_lib_file(project_parent_folder: &Path, project_name: &str) -> bool {
        let project_folder = project_parent_folder.join(project_name);
        let lib_file_path = project_folder.join(SRC_FOLDER_NAME).join("lib.rs");
        let content = fill_lib_template(project_name);
        let Ok(mut file) = std::fs::File::create(lib_file_path) else {
            return false;
        };
        match file.write_fmt(format_args!("{}", content)) {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }
}

fn get_cargo_toml_template() -> &'static str {
    return r#"
[package]
name = "@name@"
version = "0.1.0"
edition = "2021"

[features]
default = ["standalone"]
renderdoc = ["rs_render/renderdoc", "rs_engine/renderdoc"]

[dependencies]
egui = { version = "0.26.1" }
log = "0.4.17"
glam = { version = "0.22.0" }
uuid = { version = "1.6.1", features = ["v4", "fast-rng", "macro-diagnostics", "serde"] }
rs_engine = { version = "0.1.0", path = "@engine_path@/rs_engine" }
rs_render = { version = "0.1.0", path = "@engine_path@/rs_render" }

[lib]
crate-type = ["cdylib"]
    "#;
}

fn get_lib_template() -> &'static str {
    return r#"
use rs_engine::{plugin::Plugin, plugin_context::PluginContext};
use std::sync::{Arc, Mutex};

pub struct MyPlugin {
    plugin_context: Arc<Mutex<PluginContext>>,
}

impl Plugin for MyPlugin {
    fn tick(&mut self) {
        let plugin_context = self.plugin_context.clone();
        let context = &plugin_context.lock().unwrap().context;
        egui::Window::new("Plugin").show(context, |ui| {
            ui.label(format!("Time: {:?}", std::time::Instant::now()));
        });
    }

    fn unload(&mut self) {}
}

#[no_mangle]
pub fn from(plugin_context: Arc<Mutex<PluginContext>>) -> Box<dyn Plugin> {
    Box::new(MyPlugin { plugin_context })
}
    "#;
}

fn fill_lib_template(name: &str) -> String {
    let mut template = get_lib_template().to_string();
    template = template.replace("@name@", name);
    template
}

fn fill_cargo_toml_template(name: &str, engine_path: &str) -> String {
    let mut template = get_cargo_toml_template().to_string();
    template = template.replace("@name@", name);
    template = template.replace("@engine_path@", engine_path);
    template
}
