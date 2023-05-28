use wgpu::TextureFormat;

#[repr(C)]
#[derive(Debug)]
pub enum NativeWGPUTextureFormat {
    Rgba8unorm,
    Rgba8unormSrgb,
    Bgra8unorm,
    Bgra8unormSrgb,
}

impl NativeWGPUTextureFormat {
    pub fn to_texture_format(&self) -> TextureFormat {
        match self {
            NativeWGPUTextureFormat::Rgba8unorm => TextureFormat::Rgba8Unorm,
            NativeWGPUTextureFormat::Rgba8unormSrgb => TextureFormat::Rgba8UnormSrgb,
            NativeWGPUTextureFormat::Bgra8unorm => TextureFormat::Bgra8Unorm,
            NativeWGPUTextureFormat::Bgra8unormSrgb => TextureFormat::Bgra8UnormSrgb,
        }
    }
}
