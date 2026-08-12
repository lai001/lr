pub mod compressed_texture;

use rs_artifact_types::asset::Asset;

pub trait DeriveData: Asset {
    fn source_url<'a>(&'a self) -> &'a url::Url;
}
