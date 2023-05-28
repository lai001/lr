use wgpu::TextureView;

#[repr(C)]
#[derive(Debug)]
pub struct NativeWGPUTextureView {
    pub texture_view: *mut TextureView,
}
