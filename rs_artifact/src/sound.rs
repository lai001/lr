use crate::{asset::Asset, resource_type::EResourceType};
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

impl Asset for Sound {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Sound
    }
}
