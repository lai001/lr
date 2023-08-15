#[derive(Debug, Clone, Copy)]
pub struct VirtualTextureConfiguration {
    pub physical_texture_size: u32,
    pub virtual_texture_size: u32,
    pub tile_size: u32,
}

impl VirtualTextureConfiguration {
    pub fn get_max_mipmap_level(&self) -> u8 {
        1 + self.tile_size.ilog2() as u8
    }
}
