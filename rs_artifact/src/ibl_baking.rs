use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IBLBaking {
    pub name: String,
    pub url: url::Url,
    pub brdf_data: Vec<u8>,
    pub pre_filter_data: Vec<u8>,
    pub irradiance_data: Vec<u8>,
}

crate::impl_asset!(IBLBaking);
