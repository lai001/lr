use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum EEndianType {
    Big,
    Little,
    Native,
}

impl Default for EEndianType {
    fn default() -> Self {
        Self::Little
    }
}
