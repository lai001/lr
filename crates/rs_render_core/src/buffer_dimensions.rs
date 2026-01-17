#[cfg(feature = "wgpu26")]
use wgpu26 as wgpu;

// https://github.dev/gfx-rs/wgpu/blob/trunk/wgpu/examples/capture/main.rs
#[derive(Debug)]
pub struct BufferDimensions {
    pub width: usize,
    padded_width: usize,
    pub height: usize,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: usize,
}

impl BufferDimensions {
    pub fn new(width: usize, height: usize, bytes_per_pixel: usize) -> Self {
        // let bytes_per_pixel = std::mem::size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        let padded_width = padded_bytes_per_row / bytes_per_pixel;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
            padded_width,
        }
    }

    pub fn from_texture_format(
        width: usize,
        height: usize,
        texture_format: &wgpu::TextureFormat,
    ) -> Option<BufferDimensions> {
        let not_supported = vec![
            wgpu::TextureFormat::Depth24PlusStencil8,
            wgpu::TextureFormat::Depth32FloatStencil8,
            wgpu::TextureFormat::NV12,
            wgpu::TextureFormat::P010,
        ];
        if not_supported.contains(texture_format) {
            return None;
        }
        let block_copy_size = texture_format.block_copy_size(None)? as usize;
        let components = texture_format.components_with_aspect(wgpu::TextureAspect::All) as usize;
        let bytes_per_pixel = block_copy_size / components;
        Some(Self::new(width, height, bytes_per_pixel))
    }

    pub fn get_padded_width(&self) -> usize {
        self.padded_width
    }
}
