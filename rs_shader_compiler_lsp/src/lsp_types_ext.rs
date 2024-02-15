use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderPreviewParams {
    pub shader_file_path: String,
}

#[derive(Debug, Default, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderPreviewResult {
    pub code: Option<String>,
}

pub mod request {
    use super::{ShaderPreviewParams, ShaderPreviewResult};
    use lsp_types::request::Request;

    #[derive(Debug)]
    pub enum ShaderPreview {}

    impl Request for ShaderPreview {
        type Params = ShaderPreviewParams;
        type Result = ShaderPreviewResult;
        const METHOD: &'static str = "shader_compiler/ShaderPreview";
    }
}
