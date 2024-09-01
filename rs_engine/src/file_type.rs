use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum EFileType {
    Fbx,
    Glb,
    Jpeg,
    Jpg,
    Png,
    Exr,
    Hdr,
    Blend,
    Dae,
    Mp4,
    WAV,
    MP3,
}

impl ToString for EFileType {
    fn to_string(&self) -> String {
        self.to_str().to_string()
    }
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
            "jpg" => Some(EFileType::Jpg),
            "png" => Some(EFileType::Png),
            "exr" => Some(EFileType::Exr),
            "hdr" => Some(EFileType::Hdr),
            "glb" => Some(EFileType::Glb),
            "blend" => Some(EFileType::Blend),
            "dae" => Some(EFileType::Dae),
            "mp4" => Some(EFileType::Mp4),
            "wav" => Some(EFileType::WAV),
            "mp3" => Some(EFileType::MP3),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            EFileType::Fbx => "fbx",
            EFileType::Glb => "glb",
            EFileType::Jpeg => "jpeg",
            EFileType::Png => "png",
            EFileType::Exr => "exr",
            EFileType::Hdr => "hdr",
            EFileType::Jpg => "jpg",
            EFileType::Blend => "blend",
            EFileType::Dae => "dae",
            EFileType::Mp4 => "mp4",
            EFileType::WAV => "wav",
            EFileType::MP3 => "mp3",
        }
    }
}
