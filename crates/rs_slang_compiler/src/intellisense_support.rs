use crate::CompileOptions;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Intellisense {
    folders: Vec<HashMap<String, String>>,
    settings: HashMap<String, Vec<String>>,
}

pub fn save(
    compile_options: &CompileOptions,
    output_path: &Path,
    folders: HashSet<PathBuf>,
) -> crate::error::Result<()> {
    let folders = folders
        .iter()
        .map(|folder| HashMap::from([("path".to_string(), folder.display().to_string())]))
        .collect::<Vec<HashMap<String, String>>>();
    let mut settings: HashMap<String, Vec<String>> = HashMap::new();

    for def in &compile_options.definitions {
        settings
            .entry("slang.predefinedMacros".to_string())
            .or_default()
            .push(def.to_string());
    }

    for include in &compile_options.includes {
        settings
            .entry("slang.additionalSearchPaths".to_string())
            .or_default()
            .push(include.display().to_string());
    }
    let intellisense = Intellisense {
        folders: folders,
        settings: settings,
    };
    let contents = serde_json::to_string_pretty(&intellisense)?;
    std::fs::write(output_path, contents)?;
    Ok(())
}
