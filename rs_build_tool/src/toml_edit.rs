use anyhow::anyhow;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use std::path::Path;
use toml_edit::*;

pub fn enable_dylib_file(path: &Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)?;
    let mut doc = content.parse::<DocumentMut>()?;
    enable_dylib_document_mut(&mut doc);
    std::fs::write(path, doc.to_string())?;
    Ok(())
}

pub fn disable_dylib_file(path: &Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)?;
    let mut doc = content.parse::<DocumentMut>()?;
    disable_dylib_document_mut(&mut doc);
    std::fs::write(path, doc.to_string())?;
    Ok(())
}

pub fn enable_dylib_document_mut(doc: &mut DocumentMut) {
    doc["lib"] = toml_edit::Item::Table({
        let mut array = Array::default();
        array.push("dylib");
        let mut table = Table::new();
        table["crate-type"] = value(array);
        table
    });

    fix_dylib_document_mut(doc);
}

pub fn disable_dylib_document_mut(doc: &mut DocumentMut) {
    doc["lib"] = toml_edit::Item::None;
    doc["profile"] = toml_edit::Item::None;
}

pub fn fix_dylib_document_mut(doc: &mut DocumentMut) {
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
}

pub fn add_plugin_dependencies_document_mut(
    doc: &mut DocumentMut,
    name: &str,
    lib_path: &Path,
) -> anyhow::Result<()> {
    let table = doc["dependencies"]
        .as_table_mut()
        .ok_or(anyhow!("No dependencies"))?;
    let mut attributes = Table::default();
    attributes["path"] = value(
        lib_path
            .canonicalize_slash()?
            .to_str()
            .ok_or(anyhow!("Incorrect path"))?,
    );
    table[name] = toml_edit::Item::Table(attributes);
    table[name].make_value();
    Ok(())
}

pub fn remove_plugin_dependencies_document_mut(doc: &mut DocumentMut, name: &str) {
    doc["dependencies"][name] = Item::None;
}

pub fn add_plugin_dependencies_file(
    path: &Path,
    name: &str,
    lib_path: &Path,
) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)?;
    let mut doc = content.parse::<DocumentMut>()?;
    add_plugin_dependencies_document_mut(&mut doc, name, lib_path)?;
    std::fs::write(path, doc.to_string())?;
    Ok(())
}

pub fn remove_plugin_dependencies_file(path: &Path, name: &str) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)?;
    let mut doc = content.parse::<DocumentMut>()?;
    remove_plugin_dependencies_document_mut(&mut doc, name);
    std::fs::write(path, doc.to_string())?;
    Ok(())
}

pub fn add_network_feature(doc: &mut DocumentMut, project_name: &str) -> anyhow::Result<()> {
    let network_items = doc["features"]["network"]
        .as_array_mut()
        .ok_or(anyhow!("Not a array"))?;
    let item = format!("{}/network", project_name);
    let mut is_contains = false;
    for network_item in network_items.iter() {
        if let Some(s) = network_item.as_str() {
            if s == item {
                is_contains = true;
                break;
            }
        }
    }
    if !is_contains {
        network_items.push(item);
    }
    Ok(())
}

pub fn remove_network_feature(doc: &mut DocumentMut, project_name: &str) -> anyhow::Result<()> {
    let network_items = doc["features"]["network"]
        .as_array_mut()
        .ok_or(anyhow!("Not a array"))?;
    let item = format!("{}/network", project_name);
    network_items.retain(|x| {
        if let Some(s) = x.as_str() {
            s != item
        } else {
            true
        }
    });
    Ok(())
}

pub fn file_remove_network_feature(path: &Path, project_name: &str) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let mut doc = contents.parse::<DocumentMut>()?;
    remove_network_feature(&mut doc, project_name)?;
    std::fs::write(path, doc.to_string())?;
    Ok(())
}

pub fn change_edition(doc: &mut DocumentMut, edition: i32) -> anyhow::Result<()> {
    let current_edition = doc["package"]["edition"]
        .as_value_mut()
        .ok_or(anyhow!("Not a value"))?;
    *current_edition = Value::String(Formatted::new(edition.to_string()));
    Ok(())
}

pub fn change_edition_file(path: &Path, edition: i32) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let mut doc = contents.parse::<DocumentMut>()?;
    change_edition(&mut doc, edition)?;
    std::fs::write(path, doc.to_string())?;
    Ok(())
}

pub fn change_dependency_version(
    doc: &mut DocumentMut,
    crate_name: &str,
    version: &str,
) -> anyhow::Result<()> {
    let create_dependency = &mut doc["dependencies"][crate_name];
    if create_dependency.is_str() {
        *create_dependency =
            toml_edit::Item::Value(Value::String(Formatted::new(version.to_string())));
    } else if create_dependency.is_inline_table() {
        if let Some(table) = create_dependency.as_inline_table_mut() {
            if table.contains_key("version") {
                table["version"] = Value::String(Formatted::new(version.to_string()));
            }
        }
    }
    Ok(())
}

pub fn change_dependency_version_file(
    path: &Path,
    crate_name: &str,
    version: &str,
) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let mut doc = contents.parse::<DocumentMut>()?;
    change_dependency_version(&mut doc, crate_name, version)?;
    std::fs::write(path, doc.to_string())?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::toml_edit::{add_network_feature, remove_network_feature};

    #[test]
    fn test_case() {
        let contents = r#"[features]
network = ["dep:rs_network"]"#;
        let mut doc = contents.parse::<toml_edit::DocumentMut>().unwrap();
        add_network_feature(&mut doc, "rs_engine").unwrap();
        assert_eq!(
            doc.to_string().trim(),
            r#"[features]
network = ["dep:rs_network", "rs_engine/network"]"#
        );

        remove_network_feature(&mut doc, "rs_engine").unwrap();
        assert_eq!(doc.to_string().trim(), contents);
    }

    #[test]
    fn change_edition() {
        let contents = r#"[package]
edition = "2021""#;
        let target_edition: i32 = 2024;
        let mut doc = contents.parse::<toml_edit::DocumentMut>().unwrap();
        crate::toml_edit::change_edition(&mut doc, target_edition).unwrap();
        assert_eq!(
            doc.to_string().trim(),
            format!(
                r#"[package]
edition = "{}""#,
                target_edition
            )
        );
    }

    #[test]
    fn change_dependency_version_case0() {
        let contents = r#"[dependencies]
test = { version = "0.0.1", features = ["derive"] }"#;
        let target_version = "1.0.0";
        let mut doc = contents.parse::<toml_edit::DocumentMut>().unwrap();
        crate::toml_edit::change_dependency_version(&mut doc, "test", target_version).unwrap();
        assert_eq!(
            doc.to_string().trim(),
            format!(
                r#"[dependencies]
test = {{ version = "{}", features = ["derive"] }}"#,
                target_version
            )
        );
    }

    #[test]
    fn change_dependency_version_case1() {
        let contents = r#"[dependencies]
test1 = "0.0.2""#;
        let target_version = "1.0.0";
        let mut doc = contents.parse::<toml_edit::DocumentMut>().unwrap();
        crate::toml_edit::change_dependency_version(&mut doc, "test1", target_version).unwrap();
        assert_eq!(
            doc.to_string().trim(),
            format!(
                r#"[dependencies]
test1 = "{}""#,
                target_version
            )
        );
    }
}
