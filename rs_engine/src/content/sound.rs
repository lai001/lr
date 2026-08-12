use crate::{build_asset_url, url_extension::UrlExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetInfo {
    pub relative_path: PathBuf,
}

impl AssetInfo {
    pub fn get_url(&self) -> url::Url {
        build_asset_url(format!(
            "{}",
            self.relative_path.as_os_str().to_string_lossy().to_string(),
        ))
        .unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sound {
    pub url: url::Url,
    pub asset_info: AssetInfo,
}
crate::impl_content!(Sound);

impl Sound {
    pub fn new(url: url::Url, relative_path: PathBuf) -> Sound {
        Sound {
            url,
            asset_info: AssetInfo { relative_path },
        }
    }

    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }
}
