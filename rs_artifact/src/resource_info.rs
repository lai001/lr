use crate::resource_type::EResourceType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceInfo {
    pub url: url::Url,
    pub resource_type: EResourceType,
    pub offset: u64,
    pub length: u64,
}
