use crate::{asset::Asset, derive_data::DeriveData, resource_type::EResourceType};
use rs_core_minimal::file_type::TextureFileType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CompressedTexture {
    pub url: url::Url,
    pub source_url: url::Url,
    pub data: Vec<u8>,
    pub ty: TextureFileType,
}

impl Asset for CompressedTexture {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::DeriveData
    }
}

impl DeriveData for CompressedTexture {
    fn source_url<'a>(&'a self) -> &'a url::Url {
        &self.source_url
    }
}
