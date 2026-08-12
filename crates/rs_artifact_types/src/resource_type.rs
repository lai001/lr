use crate::asset::Asset;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, PartialEq, Eq, Debug, Hash, Deserialize, Serialize)]
pub struct EResourceType {
    ty_name: Cow<'static, str>,
    kind: Cow<'static, str>,
}

impl EResourceType {
    pub fn from_type<T: Asset + ?Sized>(kind: Cow<'static, str>) -> EResourceType {
        let ty_name = std::any::type_name::<T>();
        EResourceType {
            ty_name: std::borrow::Cow::Borrowed(ty_name),
            kind,
        }
    }

    pub fn from<T: Asset + ?Sized>(asset: &T, kind: Cow<'static, str>) -> EResourceType {
        let _ = asset;
        let ty_name = std::any::type_name::<T>();
        EResourceType {
            ty_name: std::borrow::Cow::Borrowed(ty_name),
            kind,
        }
    }

    pub fn ty_name(&self) -> &str {
        &self.ty_name
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }
}
