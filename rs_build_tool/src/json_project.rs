use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

pub const EDITION: &str = "2021";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dep {
    #[serde(rename = "crate")]
    pub crate_index: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Crate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_module: Option<String>,
    pub edition: String,
    pub deps: Vec<Dep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_workspace_member: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<(Vec<String>, Vec<String>)>,
    pub cfg: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub env: HashMap<String, String>,
    pub is_proc_macro: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proc_macro_dylib_path: Option<String>,
}

impl Crate {
    pub fn new(display_name: String, root_module: String, cfg: Vec<String>) -> Crate {
        Crate {
            display_name: Some(display_name),
            root_module: Some(root_module),
            edition: EDITION.to_string(),
            deps: vec![],
            is_workspace_member: None,
            source: None,
            cfg: cfg
                .iter()
                .map(|feature| format!("feature=\"{}\"", feature))
                .collect(),
            target: None,
            env: HashMap::new(),
            is_proc_macro: false,
            proc_macro_dylib_path: None,
        }
    }

    pub fn add_feature(&mut self, feature: &str) {
        self.cfg.push(format!("feature=\"{}\"", feature));
    }
}

// https://rust-analyzer.github.io/manual.html#non-cargo-based-projects
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonProject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysroot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysroot_src: Option<String>,
    pub crates: Vec<Crate>,
}

impl JsonProject {
    pub fn write_to(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::ser::to_string_pretty(self)?;
        Ok(std::fs::write(path, json)?)
    }

    pub fn solve_deps(&mut self, deps_map: HashMap<String, Vec<String>>) {
        for (crate_name, deps_names) in deps_map.iter() {
            let mut deps: Vec<Dep> = vec![];
            for deps_name in deps_names {
                let Some((i, name)) = self.crates.iter().enumerate().find(|x| {
                    x.1.display_name.clone().unwrap_or("".to_string()) == deps_name.as_str()
                }) else {
                    continue;
                };
                if let Some(name) = &name.display_name {
                    if !name.contains("-") {
                        let dep = Dep {
                            crate_index: i as i32,
                            name: name.clone(),
                        };
                        deps.push(dep);
                    }
                }
            }
            let crate_mut = self
                .crates
                .iter_mut()
                .find(|x| x.display_name.clone().unwrap_or("".to_string()) == crate_name.as_str());

            let Some(crate_mut) = crate_mut else {
                continue;
            };
            deps.sort_by(|lhs, rhs| lhs.crate_index.cmp(&rhs.crate_index));
            crate_mut.deps = deps;
        }
    }
}
