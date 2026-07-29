use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct MaterialOptions {
    pub is_skin: bool,
}

impl MaterialOptions {
    pub fn all() -> Vec<MaterialOptions> {
        vec![
            MaterialOptions { is_skin: true },
            MaterialOptions { is_skin: false },
        ]
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub enum EBlendModeType {
    #[default]
    Opaque,
    Transparent,
}

#[derive(Debug, Clone)]
pub struct RenderPipelineOptions {
    pub blend_mode: EBlendModeType,
}
