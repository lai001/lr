use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EFileType {
    Fbx,
    Jpeg,
    Png,
}

impl EFileType {
    pub fn from_path(path: &Path) -> Option<EFileType> {
        if let Some(ext) = path.extension() {
            if let Some(str) = ext.to_str() {
                Self::from_str(str)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn from_str(str: &str) -> Option<EFileType> {
        match str.to_lowercase().as_str() {
            "fbx" => Some(EFileType::Fbx),
            "jpeg" => Some(EFileType::Jpeg),
            "jpg" => Some(EFileType::Jpeg),
            "png" => Some(EFileType::Png),
            _ => None,
        }
    }
}
