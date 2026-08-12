use downcast_rs::{Downcast, impl_downcast};
use rs_artifact_types::asset::Asset;
use rs_core_minimal::types::{TypedRcRefCellBox, TypedRcRefCellBoxWeak};

pub const CONTENT_ASSET_KIND: &'static str = "content";

#[typetag::serde(tag = "type", content = "content")]
pub trait Content: erased_serde::Serialize + Downcast + Asset {
    fn get_type_text(&self) -> &'static str;
    fn get_name(&self) -> String;
    fn set_name(&mut self, new_name: String);
}

impl_downcast!(Content);

impl Asset for Box<dyn Content> {
    #[doc(hidden)]
    fn typetag_name(&self) -> &'static str {
        Content::typetag_name(self.as_ref())
    }

    #[doc(hidden)]
    fn typetag_deserialize(&self) {
        Content::typetag_deserialize(self.as_ref())
    }

    fn get_url(&self) -> url::Url {
        self.as_ref().get_url()
    }
}

pub type TypedContent<T> = TypedRcRefCellBox<dyn Content, T>;
pub type TypedContentWeak<T> = TypedRcRefCellBoxWeak<dyn Content, T>;
