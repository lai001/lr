#[derive(Debug, Clone, Copy)]
pub struct VirtualTextureConfiguration {
    pub physical_texture_size: u32,
    pub virtual_texture_size: u32,
    pub tile_size: u32,
}
