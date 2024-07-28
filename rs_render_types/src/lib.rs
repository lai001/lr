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
