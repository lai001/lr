use crate::derive_data::DeriveData;
use rs_core_minimal::file_type::TextureFileType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CompressedTexture {
    pub url: url::Url,
    pub source_url: url::Url,
    pub data: Vec<u8>,
    pub ty: TextureFileType,
}

crate::impl_asset!(CompressedTexture);

impl DeriveData for CompressedTexture {
    fn source_url<'a>(&'a self) -> &'a url::Url {
        &self.source_url
    }
}
