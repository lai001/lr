use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct TileIndex {
    pub x: u16,
    pub y: u16,
    pub mipmap_level: u32,
}
