use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaSource {
    pub url: url::Url,
    pub asset_url: url::Url,
}
