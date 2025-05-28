use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum EEndianType {
    Big,
    Little,
    Native,
}

impl EEndianType {
    pub fn current() -> EEndianType {
        #[cfg(target_endian = "big")]
        return EEndianType::Big;
        #[cfg(target_endian = "little")]
        return EEndianType::Little;
    }

    pub fn try_from_u8_value(value: u8) -> Option<EEndianType> {
        match value {
            0 => Some(EEndianType::Big),
            1 => Some(EEndianType::Little),
            2 => Some(EEndianType::Native),
            _ => None,
        }
    }
}

impl Default for EEndianType {
    fn default() -> Self {
        Self::Little
    }
}
