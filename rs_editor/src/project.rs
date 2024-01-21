use crate::{error::Result, level::Level};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub const PROJECT_FILE_EXTENSION: &str = "rsproject";
pub const ASSET_FOLDER_NAME: &str = "asset";
pub const VERSION_STR: &str = "0.0.1";

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub version_str: String,
    pub project_name: String,
    pub level: Level,
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
            level: Level::empty_level(),
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
        std::fs::create_dir(project_folder.join("src"))?;
        std::fs::create_dir(project_folder.join("asset"))?;
        std::fs::create_dir(project_folder.join("shader"))?;
        Ok(())
    }

    fn create_cargo_toml_file(project_parent_folder: &Path, project_name: &str) -> bool {
        let project_folder = project_parent_folder.join(project_name);
        let toml_file_path = project_folder.join("Cargo.toml");
        let content = fill_cargo_toml_template(project_name);
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
        let lib_file_path = project_folder.join("src").join("lib.rs");
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

[dependencies]
egui = "0.23.0"
log = "0.4.17"

[lib]
crate-type = ["rlib", "dylib"]
    "#;
}

fn get_lib_template() -> &'static str {
    return r#"
#[no_mangle]
pub fn render(context: &egui::Context) {
    egui::Area::new("Area")
        .fixed_pos(egui::pos2(32.0, 32.0))
        .show(&context, |ui| {
            ui.label(egui::RichText::new("@name@").color(egui::Color32::WHITE))
        });
}
    "#;
}

fn fill_lib_template(name: &str) -> String {
    let mut template = get_lib_template().to_string();
    template = template.replace("@name@", name);
    template
}

fn fill_cargo_toml_template(name: &str) -> String {
    let mut template = get_cargo_toml_template().to_string();
    template = template.replace("@name@", name);
    template
}
