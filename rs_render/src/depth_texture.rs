use wgpu::Texture;

pub struct DepthTexture {
    depth_texture: Texture,
}

impl DepthTexture {
    pub fn new(
        width: u32,
        height: u32,
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> DepthTexture {
        let depth_texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: depth_texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        DepthTexture { depth_texture }
    }

    pub fn get_view(&self) -> wgpu::TextureView {
        self.depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}
