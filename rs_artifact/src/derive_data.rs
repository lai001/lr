pub mod compressed_texture;

use crate::asset::Asset;

pub trait DeriveData: Asset {
    fn source_url<'a>(&'a self) -> &'a url::Url;
}
