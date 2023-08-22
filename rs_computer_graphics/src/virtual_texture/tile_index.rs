use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct TileOffset {
    pub x: u16,
    pub y: u16,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct TileIndex {
    pub tile_offset: TileOffset,
    pub mipmap_level: u8,
}
