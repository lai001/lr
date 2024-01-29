use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureFile {
    pub name: String,
    pub url: url::Url,
    pub image_reference: Option<PathBuf>,
}

impl TextureFile {
    pub fn new(name: &str, url: url::Url) -> Self {
        Self {
            url,
            image_reference: None,
            name: name.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureFolder {
    pub name: String,
    pub url: url::Url,
    pub texture_files: Vec<TextureFile>,
    pub texture_folders: Vec<TextureFolder>,
}

impl TextureFolder {
    pub fn new(name: &str, url: url::Url) -> Self {
        Self {
            name: name.to_string(),
            texture_files: Vec::new(),
            texture_folders: Vec::new(),
            url,
        }
    }
}
