use crate::resource_type::EResourceType;
use downcast_rs::{Downcast, impl_downcast};
use std::borrow::Cow;
use std::io::{Cursor, Read, Seek};

pub const ASSET_KIND: &'static str = "asset";

pub struct ResourceEncodeTask<R>
where
    R: Seek + Read,
{
    pub url: url::Url,
    pub resource_type: EResourceType,
    pub reader: R,
}

#[typetag::serde(tag = "type", content = "asset")]
pub trait Asset: erased_serde::Serialize + Downcast {
    fn get_url(&self) -> url::Url;

    fn resource_type(&self) -> EResourceType {
        EResourceType::from(self, self.asset_kind())
    }

    fn associated_resource_type() -> EResourceType
    where
        Self: Sized,
    {
        EResourceType::from_type::<Self>(Self::associated_asset_kind())
    }

    fn asset_kind(&self) -> Cow<'static, str> {
        std::borrow::Cow::Borrowed(ASSET_KIND)
    }

    fn associated_asset_kind() -> Cow<'static, str>
    where
        Self: Sized,
    {
        std::borrow::Cow::Borrowed(ASSET_KIND)
    }

    fn build_resource_encode_task(
        &self,
        reader: Cursor<Vec<u8>>,
    ) -> ResourceEncodeTask<Cursor<Vec<u8>>> {
        ResourceEncodeTask {
            url: self.get_url(),
            resource_type: self.resource_type(),
            reader,
        }
    }
}

impl_downcast!(Asset);

impl Asset for Box<dyn Asset> {
    fn get_url(&self) -> url::Url {
        self.as_ref().get_url()
    }

    #[doc(hidden)]
    fn typetag_name(&self) -> &'static str {
        self.as_ref().typetag_name()
    }

    #[doc(hidden)]
    fn typetag_deserialize(&self) {
        self.as_ref().typetag_deserialize()
    }
}
