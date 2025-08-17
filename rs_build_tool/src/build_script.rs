use crate::{
    cli::{ProfileType, ProjectArgs},
    load_plugins::create_load_plugins_file,
    toml_edit::{
        add_network_feature, add_plugin_dependencies_document_mut, add_plugin_dependencies_file,
        disable_dylib_file, enable_dylib_file, file_remove_network_feature, fix_dylib_document_mut,
        remove_plugin_dependencies_file,
    },
};
use anyhow::anyhow;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use toml_edit::DocumentMut;

// fn support_crate_names() -> Vec<String> {
//     let crate_names = vec!["rs_engine", "rs_render"];
//     crate_names.iter().map(|x| x.to_string()).collect()
// }

pub fn make_build_script(project_args: &ProjectArgs) -> anyhow::Result<()> {
    let projcet_folder = project_args
        .project_file
        .parent()
        .ok_or(anyhow!("No parent folder."))?
        .canonicalize_slash()?;
    let project_name = project_args
        .project_file
        .file_stem()
        .ok_or(anyhow!("No project name"))?
        .to_str()
        .ok_or(anyhow!("No project name"))?;

    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();
    for name in vec!["rs_desktop_standalone", "rs_editor", "rs_android"] {
        create_load_plugins_file(name, None, true)?;
    }
    match &project_args.mode_type {
        crate::cli::ModeType::Editor => {
            let crate_names = vec!["rs_engine", "rs_render"];
            let manifest_files = crate_names
                .iter()
                .map(|x| engine_root_dir.join(x).join("Cargo.toml"));

            if project_args.is_enable_dylib {
                for path in manifest_files.clone() {
                    enable_dylib_file(&path)?;
                }

                enable_dylib_file(&projcet_folder.join("Cargo.toml"))?;
            }

            {
                let editor_manifest_file = engine_root_dir.join("rs_editor/Cargo.toml");
                let content = std::fs::read_to_string(&editor_manifest_file)?;
                let mut doc = content.parse::<DocumentMut>()?;
                add_plugin_dependencies_document_mut(&mut doc, project_name, &projcet_folder)?;
                add_network_feature(&mut doc, project_name)?;
                if project_args.is_enable_dylib {
                    fix_dylib_document_mut(&mut doc);
                }
                std::fs::write(&editor_manifest_file, doc.to_string())?;
            }
            if !project_args.is_enable_dylib {
                create_load_plugins_file("rs_editor", Some(project_name.to_string()), true)?;
                return Ok(());
            }
            let old_dir = std::env::current_dir()?;
            std::env::set_current_dir(engine_root_dir.join("rs_editor"))?;
            let mut command = std::process::Command::new("cargo");
            command
                .arg("build")
                .arg("-vv")
                .arg("--message-format")
                .arg("json")
                .arg("--color")
                .arg("never")
                .arg("--package")
                .arg("rs_editor")
                .arg("--bin")
                .arg("editor")
                .arg("--features")
                .arg("editor")
                .arg("--features")
                .arg("plugin_shared_crate");

            if project_args.profile_type == ProfileType::Release {
                command.arg("--release");
            }

            let output = command.output()?;
            if !output.status.success() {
                return Err(anyhow!(
                    "cargo build, {:?}\n{}",
                    output.status.code(),
                    String::from_utf8(output.stderr)?
                ));
            }

            let stderr = String::from_utf8(output.stderr)?;
            let lines = stderr.split("\n");
            let mut is_created = false;
            for line in lines {
                if line.contains("Running")
                    && line.contains("--crate-name")
                    && line.contains(&format!("CARGO_CRATE_NAME={}", project_name))
                {
                    let line = line.trim_start().trim_end().to_string();
                    let line = line
                        .strip_prefix("Running `")
                        .ok_or(anyhow!("strip_prefix error"))?;
                    let line = line
                        .strip_suffix("`")
                        .ok_or(anyhow!("strip_suffix error"))?;
                    let contents: String = line.replace("&& ", "\n");
                    let mut contents = contents.replace(
                        "--error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat",
                        "",
                    );
                    contents.insert_str(0, "echo off\n");
                    std::fs::write(projcet_folder.join("build.bat"), contents)?;
                    is_created = true;
                    break;
                }
            }
            if !is_created {
                return Err(anyhow!("Failed to create build.bat file"));
            }
            std::env::set_current_dir(old_dir)?;
        }
        crate::cli::ModeType::Standalone => {
            for crate_name in ["rs_desktop_standalone", "rs_android"] {
                create_load_plugins_file(crate_name, Some(project_name.to_string()), true)?;
                add_plugin_dependencies_file(
                    &engine_root_dir.join(crate_name).join("Cargo.toml"),
                    project_name,
                    &projcet_folder,
                )?;
            }
            disable_dylib_file(&projcet_folder.join("Cargo.toml"))?;
        }
    }

    Ok(())
}

pub fn clean(project_args: &ProjectArgs) -> anyhow::Result<()> {
    let projcet_folder = project_args
        .project_file
        .parent()
        .ok_or(anyhow!("No parent folder."))?
        .canonicalize_slash()?;
    let project_name = project_args
        .project_file
        .file_stem()
        .ok_or(anyhow!("No project name"))?
        .to_str()
        .ok_or(anyhow!("No project name"))?;

    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();

    let crate_names = vec!["rs_editor", "rs_engine", "rs_render"];
    let manifest_files = crate_names
        .iter()
        .map(|x| engine_root_dir.join(x).join("Cargo.toml"));

    for path in manifest_files {
        disable_dylib_file(&path)?;
    }

    let manifest_file = engine_root_dir.join("rs_editor/Cargo.toml");
    remove_plugin_dependencies_file(&manifest_file, project_name)?;
    file_remove_network_feature(&manifest_file, project_name)?;
    for crate_name in ["rs_desktop_standalone", "rs_android"] {
        let manifest_file = engine_root_dir.join(crate_name).join("Cargo.toml");
        remove_plugin_dependencies_file(&manifest_file, project_name)?;
    }
    for name in vec!["rs_desktop_standalone", "rs_editor", "rs_android"] {
        create_load_plugins_file(name, None, true)?;
    }
    let project_manifest_file = projcet_folder.join("Cargo.toml");
    disable_dylib_file(&project_manifest_file)?;
    Ok(())
}
