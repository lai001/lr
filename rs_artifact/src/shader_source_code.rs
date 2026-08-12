use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShaderSourceCode {
    pub name: String,
    pub id: uuid::Uuid,
    pub url: url::Url,
    pub code: String,
}

crate::impl_asset!(ShaderSourceCode);
