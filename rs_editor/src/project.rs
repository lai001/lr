use anyhow::anyhow;
use path_slash::PathBufExt;
use rs_artifact::EEndianType;
use rs_core_minimal::settings::Settings;
use rs_engine::content::content_file_type::EContentFileType;
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

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub version_str: String,
    pub project_name: String,
    pub settings: Rc<RefCell<Settings>>,
    pub endian_type: EEndianType,
    pub materials: Vec<Rc<RefCell<crate::material::Material>>>,
    pub content: Rc<RefCell<crate::content_folder::ContentFolder>>,
}

impl Project {
    pub fn create_empty_project(
        project_parent_folder: &Path,
        project_name: &str,
    ) -> anyhow::Result<PathBuf> {
        Self::create_empty_project_folders(project_parent_folder, project_name)?;
        Self::create_empty_project_file_to_disk(&project_parent_folder, project_name)?;
        Self::create_cargo_toml_file(project_parent_folder, project_name)?;
        Self::create_lib_file(project_parent_folder, project_name)?;
        Self::create_cargo_config_toml_file(project_parent_folder, project_name)?;
        #[cfg(any(feature = "plugin_shared_lib", feature = "plugin_shared_crate"))]
        Self::create_my_plugin_file(project_parent_folder, project_name)?;
        let project_folder = project_parent_folder.join(project_name);
        let project_file_path =
            project_folder.join(format!("{}.{}", project_name, PROJECT_FILE_EXTENSION));
        Ok(project_file_path)
    }

    fn create_empty_project_file_to_disk(
        project_parent_folder: &Path,
        project_name: &str,
    ) -> anyhow::Result<()> {
        let project_folder = project_parent_folder.join(project_name);
        let project_file_path =
            project_folder.join(format!("{}.{}", project_name, PROJECT_FILE_EXTENSION));
        if project_file_path.exists() {
            return Err(anyhow!("{:?} is exists", project_file_path));
        }
        let content = Rc::new(RefCell::new(crate::content_folder::ContentFolder::default()));
        content
            .borrow_mut()
            .files
            .push(EContentFileType::Level(Rc::new(RefCell::new(
                rs_engine::content::level::Level::empty_level(),
            ))));
        let empty_project = Project {
            version_str: VERSION_STR.to_string(),
            project_name: project_name.to_string(),
            endian_type: EEndianType::Little,
            settings: Rc::new(RefCell::new(Settings::default())),
            content,
            materials: vec![],
        };
        let json_str = serde_json::ser::to_string_pretty(&empty_project)?;
        let mut file = std::fs::File::create(project_file_path)?;
        Ok(file.write_fmt(format_args!("{}", json_str))?)
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

    fn create_cargo_toml_file(
        project_parent_folder: &Path,
        project_name: &str,
    ) -> anyhow::Result<()> {
        let current_dir = std::env::current_dir()?;
        let engien_dir = current_dir.join("../../../").canonicalize()?;
        let engien_dir = dunce::canonicalize(&engien_dir)?;
        let project_folder = project_parent_folder.join(project_name);
        let toml_file_path = project_folder.join("Cargo.toml");
        let engine_path = engien_dir.to_slash().ok_or(anyhow!(
            "Fail to convert {:?} to slash style path",
            engien_dir
        ))?;
        let content = fill_cargo_toml_template(project_name, &engine_path);
        let mut file = std::fs::File::create(toml_file_path)?;
        Ok(file.write_fmt(format_args!("{}", content))?)
    }

    fn create_lib_file(project_parent_folder: &Path, project_name: &str) -> anyhow::Result<()> {
        let project_folder = project_parent_folder.join(project_name);
        let lib_file_path = project_folder.join(SRC_FOLDER_NAME).join("lib.rs");
        let content = get_lib_template();
        let mut file = std::fs::File::create(lib_file_path)?;
        Ok(file.write_fmt(format_args!("{}", content))?)
    }

    #[cfg(any(feature = "plugin_shared_lib", feature = "plugin_shared_crate"))]
    fn create_my_plugin_file(
        project_parent_folder: &Path,
        project_name: &str,
    ) -> anyhow::Result<()> {
        let project_folder = project_parent_folder.join(project_name);
        let lib_file_path = project_folder.join(SRC_FOLDER_NAME).join("my_plugin.rs");
        let content =
            fill_my_plugin_template(project_name, rs_native_plugin::symbol_name::CREATE_PLUGIN);
        let mut file = std::fs::File::create(lib_file_path)?;
        Ok(file.write_fmt(format_args!("{}", content))?)
    }

    fn create_cargo_config_toml_file(
        project_parent_folder: &Path,
        project_name: &str,
    ) -> anyhow::Result<()> {
        let project_folder = project_parent_folder.join(project_name);
        let toml_file_path = project_folder.join(".cargo/config.toml");
        let parent = toml_file_path
            .parent()
            .ok_or(anyhow!("Parent folder not found"))?;
        if !parent.exists() {
            std::fs::create_dir(parent)?;
        }
        let content = get_cargo_config_toml_template();
        let mut file = std::fs::File::create(toml_file_path)?;
        Ok(file.write_fmt(format_args!("{}", content))?)
    }
}

fn get_cargo_config_toml_template() -> &'static str {
    return r#"[build]
rustflags = ["-C", "prefer-dynamic", "-C", "rpath"]
    "#;
}

fn get_cargo_toml_template() -> &'static str {
    return r#"[package]
name = "@name@"
version = "0.1.0"
edition = "2021"

[features]
plugin_shared_lib = ["rs_native_plugin/plugin_shared_lib"]
plugin_shared_crate = [
    "rs_native_plugin/plugin_shared_crate",
    "dep:rs_engine",
    "dep:rs_render",
]
default = ["plugin_shared_lib"]
editor = ["rs_render/editor", "rs_engine/editor"]
standalone = ["rs_render/standalone", "rs_engine/standalone"]
profiler = ["rs_render/default"]
renderdoc = ["rs_render/renderdoc"]

[dependencies]
rs_engine = { path = "@engine_path@/rs_engine", optional = true }
rs_render = { path = "@engine_path@/rs_render", optional = true }
rs_native_plugin = { path = "@engine_path@/rs_native_plugin", default_features = false }

[lib]
crate-type = ["cdylib"]
    "#;
}

#[cfg(any(feature = "plugin_shared_lib", feature = "plugin_shared_crate"))]
fn get_my_plugin_template() -> &'static str {
    return r#"use rs_native_plugin::plugin::*;

pub struct MyPlugin {}

impl Plugin for MyPlugin {
    #[cfg(feature = "plugin_shared_lib")]
    fn tick(&mut self, engine: Engine) {
        unsafe {
            let mode = 0;
            rs_engine_Engine_set_view_mode(engine, mode);
        }
    }

    #[cfg(feature = "plugin_shared_crate")]
    fn tick(&mut self, engine: &mut rs_engine::engine::Engine) {
        engine.set_view_mode(rs_render::view_mode::EViewModeType::Wireframe);
    }
}

#[no_mangle]
pub fn @symbol_name@() -> Box<dyn Plugin> {
    let plugin = MyPlugin {};
    Box::new(plugin)
}
    "#;
}

fn get_lib_template() -> &'static str {
    return r#"pub mod my_plugin;
    "#;
}

#[cfg(any(feature = "plugin_shared_lib", feature = "plugin_shared_crate"))]
fn fill_my_plugin_template(name: &str, symbol_name: &str) -> String {
    let mut template = get_my_plugin_template().to_string();
    template = template.replace("@name@", name);
    template = template.replace("@symbol_name@", symbol_name);
    template
}

fn fill_cargo_toml_template(name: &str, engine_path: &str) -> String {
    let mut template = get_cargo_toml_template().to_string();
    template = template.replace("@name@", name);
    template = template.replace("@engine_path@", engine_path);
    template
}
