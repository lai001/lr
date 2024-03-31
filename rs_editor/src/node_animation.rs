use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeAnimation {
    pub name: String,
    pub url: url::Url,
    pub asset_reference: String,
}
