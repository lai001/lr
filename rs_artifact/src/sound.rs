use serde::{Deserialize, Serialize};

#[derive(Copy, PartialEq, Eq, Debug, Clone, Hash, Serialize, Deserialize)]
pub enum ESoundFileType {
    Wav,
    Mp3,
    Ogg,
    Rgba8,
    Unknow,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sound {
    pub url: url::Url,
    pub sound_file_type: ESoundFileType,
    pub data: Vec<u8>,
}

crate::impl_asset!(Sound);
