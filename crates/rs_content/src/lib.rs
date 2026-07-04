use downcast_rs::{Downcast, impl_downcast};
use rs_core_minimal::types::{HasUrl, TypedRcRefCellBox, TypedRcRefCellBoxWeak};

#[typetag::serde(tag = "type", content = "content")]
pub trait Content: HasUrl + erased_serde::Serialize + Downcast {
    fn get_type_text(&self) -> &'static str;
    fn get_name(&self) -> String;
    fn set_name(&mut self, new_name: String);
}

impl_downcast!(Content);

pub type ContentWrapper<T> = TypedRcRefCellBox<dyn Content, T>;
pub type ContentWrapperWeak<T> = TypedRcRefCellBoxWeak<dyn Content, T>;
