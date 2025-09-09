use anyhow::anyhow;
use clap::Parser;
use rs_build_tool::{
    build_script::{clean, make_build_script},
    cli::{Cli, ProjectFilesArgs},
    json_project::{Crate, JsonProject},
    toml_edit::{change_dependency_version_file, change_edition_file},
};
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use rs_foundation::change_working_directory;
use serde_json::{json, Map, Value};
use std::{collections::HashMap, path::Path};

fn try_write_setting_json_file(settings_path: &Path) -> anyhow::Result<()> {
    if settings_path.is_file() {
        let file = std::fs::File::open(settings_path)?;
        let reader = std::io::BufReader::new(file);

        let mut value: Value = serde_json::de::from_reader(reader)?;
        match value.get_mut("rust-analyzer.linkedProjects") {
            Some(linked_projects) => {
                let rust_project_value =
                    serde_json::Value::String(".vscode/rust-project.json".to_string());
                if let Some(projects) = linked_projects.as_array_mut() {
                    if !projects.contains(&rust_project_value) {
                        projects.push(rust_project_value);
                    }
                }
            }
            None => {
                let object_mut = value
                    .as_object_mut()
                    .ok_or(anyhow!("Root is not a object"))?;
                object_mut.insert(
                    "rust-analyzer.linkedProjects".to_string(),
                    json!([".vscode/rust-project.json"]),
                );
            }
        }

        match value.get_mut("rust-analyzer.cargo.features") {
            Some(features) => {
                let rust_project_value =
                    serde_json::Value::String("plugin_shared_crate_import".to_string());
                if let Some(projects) = features.as_array_mut() {
                    if !projects.contains(&rust_project_value) {
                        projects.push(rust_project_value);
                    }
                }
            }
            None => {
                let object_mut = value
                    .as_object_mut()
                    .ok_or(anyhow!("Root is not a object"))?;
                object_mut.insert(
                    "rust-analyzer.cargo.features".to_string(),
                    json!(["plugin_shared_crate_import"]),
                );
            }
        }
        let contents = serde_json::ser::to_string_pretty(&value)?;
        std::fs::write(settings_path, contents)?;
    } else {
        let value = serde_json::json!({
            "rust-analyzer.linkedProjects": [
                ".vscode/rust-project.json"
            ],
            "rust-analyzer.cargo.features": [
                "plugin_shared_crate_import"
            ]
        });
        let contents = serde_json::ser::to_string_pretty(&value)?;
        std::fs::write(settings_path, contents)?;
    }
    Ok(())
}

fn fetch_metadata() -> anyhow::Result<Map<String, Value>> {
    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();
    let output = std::process::Command::new("cargo")
        .current_dir(engine_root_dir.join("rs_editor"))
        .arg("metadata")
        .output()?;
    if output.status.success() {
        let out = String::from_utf8(output.stdout)?;
        let value: serde_json::Value = serde_json::from_str(&out)?;
        let ovject = value.as_object().ok_or(anyhow!("Not a object"))?;
        Ok(ovject.to_owned())
    } else {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow!("{}", err));
    }
}

fn write_rust_project_json_file(
    projcet_folder: &Path,
    project_files_args: &ProjectFilesArgs,
) -> anyhow::Result<()> {
    let mut vars = std::env::vars();
    let rustup_home = vars
        .find_map(|(k, v)| {
            if k == "RUSTUP_HOME" {
                return Some(v);
            } else {
                return None;
            }
        })
        .ok_or(anyhow!("No `RUSTUP_HOME` environment variable"))?;

    let sysroot_src = Path::new(&rustup_home)
        .join("toolchains/stable-x86_64-pc-windows-msvc/lib/rustlib/src/rust/library")
        .canonicalize_slash()?;
    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();

    let project_name = project_files_args
        .project_file
        .file_stem()
        .ok_or(anyhow!("No project name"))?
        .to_str()
        .ok_or(anyhow!("No project name"))?;

    let project_crate = Crate::new(
        project_name.to_string(),
        projcet_folder
            .join("src/lib.rs")
            .canonicalize_slash()?
            .to_str()
            .ok_or(anyhow!(""))?
            .to_string(),
        vec!["plugin_shared_crate_import".to_string()],
    );

    let engine_crate = Crate::new(
        "rs_engine".to_string(),
        engine_root_dir
            .join("rs_engine/src/lib.rs")
            .canonicalize_slash()?
            .to_str()
            .ok_or(anyhow!(""))?
            .to_string(),
        vec![
            "editor".to_string(),
            "plugin_shared_crate".to_string(),
            "plugin_dotnet".to_string(),
        ],
    );

    // let render_crate = Crate::new(
    //     "rs_render".to_string(),
    //     engine_root_dir
    //         .join("rs_render/src/lib.rs")
    //         .canonicalize_slash()?
    //         .to_str()
    //         .ok_or(anyhow!(""))?
    //         .to_string(),
    //     vec![],
    // );

    // let mut proc_macros_crate = Crate::new(
    //     "rs_proc_macros".to_string(),
    //     engine_root_dir
    //         .join("rs_proc_macros/src/lib.rs")
    //         .canonicalize_slash()?
    //         .to_str()
    //         .ok_or(anyhow!(""))?
    //         .to_string(),
    //     vec![],
    // );
    // proc_macros_crate.is_proc_macro = true;

    let mut dependencies_crates: Vec<Crate> = vec![];
    let mut dependencies_names: Vec<String> = vec![];

    let output = std::process::Command::new("cargo")
        .current_dir(engine_root_dir.join("rs_editor"))
        .arg("metadata")
        .output()?;
    if output.status.success() {
        let out = String::from_utf8(output.stdout)?;
        let value: serde_json::Value = serde_json::from_str(&out)?;
        if let Some(object) = value.as_object() {
            let packages = object
                .get("packages")
                .ok_or(anyhow!("No packages"))?
                .as_array()
                .ok_or(anyhow!("No packages"))?;
            let packages = packages
                .iter()
                .flat_map(|x| x.as_object())
                .collect::<Vec<&Map<String, Value>>>();
            for package in packages.iter() {
                let name = package
                    .get("name")
                    .map(|x| x.as_str())
                    .flatten()
                    .ok_or(anyhow!("No package name"))?;
                if name != "rs_engine" {
                    continue;
                }
                let dependencies = package
                    .get("dependencies")
                    .map(|x| x.as_array())
                    .flatten()
                    .ok_or(anyhow!("No dependencies"))?;
                dependencies_names = dependencies
                    .iter()
                    .flat_map(|x| x.as_object())
                    .flat_map(|x| x.get("name"))
                    .flat_map(|x| x.as_str())
                    .map(|x| x.to_string())
                    .collect();
            }

            for package in packages {
                let name = package
                    .get("name")
                    .map(|x| x.as_str())
                    .flatten()
                    .ok_or(anyhow!("No package name"))?;
                if !dependencies_names.contains(&name.to_string()) {
                    continue;
                }
                let targets = package
                    .get("targets")
                    .ok_or(anyhow!("No targets"))?
                    .as_array()
                    .ok_or(anyhow!("No targets"))?;
                let target = targets.first().ok_or(anyhow!(""))?;
                let kind = target
                    .get("kind")
                    .map(|x| x.as_array())
                    .flatten()
                    .map(|x| x.first())
                    .flatten()
                    .ok_or(anyhow!("{target:?} No kind"))?;
                let src_path = target
                    .get("src_path")
                    .map(|x| x.as_str())
                    .flatten()
                    .ok_or(anyhow!("No src_path"))?;
                let mut dependency_crate =
                    Crate::new(name.to_string(), src_path.to_string(), vec![]);
                dependency_crate.is_proc_macro = kind == "proc-macro";
                dependencies_crates.push(dependency_crate);
            }
        }
    } else {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow!("{}", err));
    }
    dependencies_names.push(
        (&engine_crate)
            .display_name
            .clone()
            .ok_or(anyhow!(""))?
            .to_string(),
    );

    let mut crates = vec![];
    crates.append(&mut vec![project_crate, engine_crate]);
    crates.append(&mut dependencies_crates);

    let mut project = JsonProject {
        sysroot: None,
        sysroot_src: Some(sysroot_src.to_str().ok_or(anyhow!(""))?.to_string()),
        crates,
    };
    project.solve_deps(HashMap::from([(
        project_name.to_string(),
        dependencies_names,
    )]));
    let rust_project_json_path = projcet_folder.join(".vscode/rust-project.json");
    let vscode_folder = projcet_folder.join(".vscode");
    if !vscode_folder.exists() {
        std::fs::create_dir_all(vscode_folder)?;
    }
    project.write_to(&rust_project_json_path)?;
    Ok(())
}

fn make_search_line(search_dir: &str) -> String {
    format!(
        r#"println!("cargo:rustc-link-search={}", "{}");"#,
        "{}", search_dir
    )
}

fn write_build_script_file(projcet_folder: &Path) -> anyhow::Result<()> {
    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();

    let metadata = fetch_metadata()?;
    let packages = metadata
        .get("packages")
        .ok_or(anyhow!("No packages"))?
        .as_array()
        .ok_or(anyhow!("No packages"))?;
    let mut lines: Vec<String> = vec![];

    for package in packages {
        let Some(name) = package.get("name").map(|x| x.as_str()).flatten() else {
            continue;
        };

        if name != "windows_x86_64_msvc" {
            continue;
        }

        let Some(manifest_path) = package.get("manifest_path").map(|x| x.as_str()).flatten() else {
            continue;
        };
        let manifest_path = Path::new(manifest_path);
        let Some(crate_folder) = manifest_path.parent() else {
            continue;
        };
        let crate_folder = crate_folder.join("lib").canonicalize_slash()?;
        let search_dir = crate_folder
            .to_str()
            .ok_or(anyhow!("Not a valid path, {crate_folder:?}"))?;

        lines.push(make_search_line(search_dir));
    }

    let ffmpeg_search_dir = engine_root_dir
        .join(".xmake/deps/ffmpeg-n6.0-31-g1ebb0e43f9-win64-gpl-shared-6.0/lib")
        .canonicalize_slash()?;
    let ffmpeg_search_dir = ffmpeg_search_dir
        .to_str()
        .ok_or(anyhow!("Not a valid path"))?;
    lines.push(make_search_line(ffmpeg_search_dir));

    let editor_search_dir = engine_root_dir
        .join("rs_editor/target/debug")
        .canonicalize_slash()?;
    let rs_editor_search_dir = editor_search_dir
        .to_str()
        .ok_or(anyhow!("Not a valid path"))?;
    lines.push(make_search_line(rs_editor_search_dir));

    let contents = format!(
        r#"fn main() {{
{}
}}"#,
        lines.concat()
    );

    let build_script_file_path = projcet_folder.join("build.rs");
    if build_script_file_path.exists() {
        std::fs::remove_file(&build_script_file_path)?;
    }
    std::fs::write(&build_script_file_path, contents)?;
    Ok(())
}

fn generate_project_files(project_files_args: ProjectFilesArgs) -> anyhow::Result<()> {
    let projcet_folder = project_files_args
        .project_file
        .parent()
        .ok_or(anyhow!("No parent folder."))?
        .canonicalize_slash()?;
    write_rust_project_json_file(&projcet_folder, &project_files_args)?;
    let settings_path = projcet_folder.join(".vscode/settings.json");
    try_write_setting_json_file(&settings_path)?;
    write_build_script_file(&projcet_folder)?;
    Ok(())
}

fn visit_manifest_files(
    closure: &mut impl FnMut(&Path) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir().canonicalize()?;
    let engine_root_dir = engine_root_dir.to_string_lossy();
    let paths = glob::glob(&format!("{}/crates/rs_*/Cargo.toml", engine_root_dir))?
        .chain(glob::glob(&format!(
            "{}/programs/rs_*/Cargo.toml",
            engine_root_dir
        ))?)
        .chain(glob::glob(&format!("{}/rs_*/Cargo.toml", engine_root_dir))?);
    for path in paths {
        if let Ok(path) = path {
            let mut is_ignore = false;
            if let Some(Some(Some(file_stem))) =
                path.parent().map(|x| x.file_stem().map(|x| x.to_str()))
            {
                if file_stem.contains("rs_computer_graphics") {
                    is_ignore = true;
                }
            }
            if !is_ignore {
                closure(&path)?;
            }
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    change_working_directory();
    let mut builder = env_logger::Builder::new();
    builder.write_style(env_logger::WriteStyle::Auto);
    builder.filter_level(log::LevelFilter::Trace);
    builder.init();

    let cli = Cli::parse();
    match cli {
        Cli::ProjectFiles(project_files_args) => generate_project_files(project_files_args)?,
        Cli::Project(project_args) => {
            if project_args.is_enable {
                make_build_script(&project_args)?;
            } else {
                clean(&project_args)?;
            }
        }
        Cli::CreateDefaultLoadPluginsFile => {
            for name in vec!["rs_desktop_standalone", "rs_editor"] {
                rs_build_tool::load_plugins::create_load_plugins_file(name, None, false)?;
            }
        }
        Cli::UpdateEdition => {
            visit_manifest_files(&mut |path| change_edition_file(path, 2024))?;
        }
        Cli::UpdateDependencies(update_dependencies_args) => {
            let crate_name = &update_dependencies_args.crate_name;
            let version = &update_dependencies_args.crate_version;
            match &update_dependencies_args.manifest_file {
                Some(manifest_file) => {
                    change_dependency_version_file(manifest_file, crate_name, version)?;
                }
                None => {
                    visit_manifest_files(&mut |path| {
                        let _ = change_dependency_version_file(path, crate_name, version);
                        Ok(())
                    })?;
                }
            }
        }
    }

    Ok(())
}
