use crate::cli::HotreloadArgs;
use anyhow::anyhow;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use toml_edit::{value, Array, DocumentMut, Item, Table};

fn support_crate_names() -> Vec<String> {
    let crate_names = vec!["rs_engine", "rs_render", "rs_native_plugin"];
    crate_names.iter().map(|x| x.to_string()).collect()
}

pub fn make_build_script(hotreload_args: &HotreloadArgs) -> anyhow::Result<()> {
    let projcet_folder = hotreload_args
        .project_file
        .parent()
        .ok_or(anyhow!("No parent folder."))?
        .canonicalize_slash()?;
    let project_name = hotreload_args
        .project_file
        .file_stem()
        .ok_or(anyhow!("No project name"))?
        .to_str()
        .ok_or(anyhow!("No project name"))?;

    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();

    let crate_names = support_crate_names();
    let manifest_files = crate_names
        .iter()
        .map(|x| engine_root_dir.join(x).join("Cargo.toml"));

    for path in manifest_files.clone() {
        let content = std::fs::read_to_string(&path)?;
        let mut doc = content.parse::<DocumentMut>()?;

        doc["lib"] = toml_edit::Item::Table({
            let mut array = Array::default();
            array.push("dylib");
            let mut table = Table::new();
            table["crate-type"] = value(array);
            table
        });

        doc["profile"] = toml_edit::Item::Table({
            let mut level = Table::new();
            level["opt-level"] = value(2);

            let mut any = Table::default();
            any["*"] = toml_edit::Item::Table(level);
            any.set_dotted(true);

            let mut package = Table::default();
            package["package"] = toml_edit::Item::Table(any);
            package.set_dotted(true);

            let mut dev = Table::default();
            dev["dev"] = toml_edit::Item::Table(package);
            dev.set_dotted(true);
            dev
        });

        std::fs::write(&path, doc.to_string())?;
    }

    {
        let editor_manifest_file = engine_root_dir.join("rs_editor/Cargo.toml");
        let content = std::fs::read_to_string(&editor_manifest_file)?;
        let mut doc = content.parse::<DocumentMut>()?;

        let table = doc["dependencies"].as_table_mut().ok_or(anyhow!("No dependencies"))?;
        let mut attributes = Table::default();
        attributes["path"] = value(projcet_folder.canonicalize_slash()?.to_str().unwrap());
        table[project_name] = toml_edit::Item::Table(attributes);
        table[project_name].make_value();

        doc["profile"] = toml_edit::Item::Table({
            let mut level = Table::new();
            level["opt-level"] = value(2);

            let mut any = Table::default();
            any["*"] = toml_edit::Item::Table(level);
            any.set_dotted(true);

            let mut package = Table::default();
            package["package"] = toml_edit::Item::Table(any);
            package.set_dotted(true);

            let mut dev = Table::default();
            dev["dev"] = toml_edit::Item::Table(package);
            dev.set_dotted(true);
            dev
        });

        std::fs::write(&editor_manifest_file, doc.to_string())?;
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
        .arg("plugin_shared_crate_export");

    let output = command.output()?;
    if !output.status.success() {
        return Err(anyhow!("cargo build, {:?}\n{}", output.status.code(), String::from_utf8(output.stderr)?));
    }

    let stderr = String::from_utf8(output.stderr)?;
    let lines = stderr.split("\n");
    for line in lines {
        if line.contains("Running")
            && line.contains("--crate-name")
            && line.contains(&format!("CARGO_CRATE_NAME={}", project_name))
        {
            let line = line.trim_start().trim_end().to_string();
            let line = line.strip_prefix("Running `").ok_or(anyhow!("strip_prefix error"))?;
            let line = line.strip_suffix("`").ok_or(anyhow!("strip_suffix error"))?;
            let contents: String = line.replace("&& ", "\n");
            let mut contents = contents.replace(
                "--error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat",
                "",
            );
            contents.insert_str(0, "echo off\n");
            std::fs::write(projcet_folder.join("build.bat"), contents)?;
            break;
        }
    }

    std::env::set_current_dir(old_dir)?;

    Ok(())
}

pub fn clean(hotreload_args: &HotreloadArgs) -> anyhow::Result<()> {
    let project_name = hotreload_args
        .project_file
        .file_stem()
        .ok_or(anyhow!("No project name"))?
        .to_str()
        .ok_or(anyhow!("No project name"))?;

    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();

    let crate_names = support_crate_names();
    let manifest_files = crate_names
        .iter()
        .map(|x| engine_root_dir.join(x).join("Cargo.toml"));

    for path in manifest_files {
        let content: String = std::fs::read_to_string(&path)?;
        let mut doc = content.parse::<DocumentMut>()?;
        doc.remove_entry("lib");
        doc.remove_entry("profile");
        std::fs::write(&path, doc.to_string())?;
    }

    {
        let editor_manifest_file = engine_root_dir.join("rs_editor/Cargo.toml");
        let content = std::fs::read_to_string(&editor_manifest_file)?;
        let mut doc = content.parse::<DocumentMut>()?;
        doc["dependencies"][project_name] = Item::None;
        doc["profile"] = Item::None;
        std::fs::write(&editor_manifest_file, doc.to_string())?;
    }
    Ok(())
}
