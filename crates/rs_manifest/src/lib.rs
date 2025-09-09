use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use toml_edit::DocumentMut;

pub struct CargoManifest {
    path: PathBuf,
    doc: DocumentMut,
}

impl CargoManifest {
    pub fn new(path: PathBuf) -> Option<Self> {
        let doc_mut = read_toml(&path)?;
        Some(Self { path, doc: doc_mut })
    }

    pub fn from_string(string: &str) -> Option<Self> {
        let doc_mut = string.parse::<DocumentMut>().ok()?;
        Some(Self {
            path: PathBuf::new(),
            doc: doc_mut,
        })
    }

    pub fn save(&self) -> bool {
        std::fs::write(&self.path, self.doc.to_string()).is_ok()
    }

    pub fn save_to(&self, path: &Path) -> bool {
        std::fs::write(path, self.doc.to_string()).is_ok()
    }

    pub fn doc_mut(&mut self) -> &mut DocumentMut {
        &mut self.doc
    }

    pub fn read_create_version(&mut self, versions: &mut HashMap<&str, String>) {
        let Some(Some(dependencies)) = self.doc.get("dependencies").map(|x| x.as_table_like())
        else {
            return;
        };
        for (name, version) in versions {
            if let Some(Some(table)) = dependencies.get(&name).map(|x| x.as_table_like()) {
                if let Some(Some(v)) = table.get("version").map(|x| x.as_str()) {
                    *version = v.to_string();
                }
            } else if let Some(Some(v)) = dependencies.get(&name).map(|x| x.as_str()) {
                *version = v.to_string();
            }
        }
    }
}

pub fn read_toml(path: &Path) -> Option<DocumentMut> {
    let content = std::fs::read_to_string(path).ok()?;
    let doc = content.parse::<DocumentMut>().ok()?;
    Some(doc)
}

#[cfg(test)]
mod test {
    use crate::CargoManifest;
    use std::collections::HashMap;

    #[test]
    fn test_case() {
        let contents = r#"[dependencies]
v8 = "140.0.0"
anyhow = { version = "1.0.99" }"#;
        let mut cargo_manifest = CargoManifest::from_string(contents).unwrap();
        let mut versions = HashMap::from([("v8", "".to_string()), ("anyhow", "".to_string())]);
        cargo_manifest.read_create_version(&mut versions);
        assert_eq!(versions["v8"], "140.0.0");
        assert_eq!(versions["anyhow"], "1.0.99");
    }
}
