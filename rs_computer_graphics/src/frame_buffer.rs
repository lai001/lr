use wgpu::TextureFormat;
use winit::dpi::PhysicalSize;

use crate::{buffer_dimensions::BufferDimensions, depth_texture::DepthTexture};

pub struct FrameBuffer {
    color_texture: wgpu::Texture,
    depth_texture: DepthTexture,
}

impl FrameBuffer {
    pub fn new(
        device: &wgpu::Device,
        sieze: PhysicalSize<u32>,
        color_format: TextureFormat,
    ) -> FrameBuffer {
        let available_texture_formats = std::collections::HashMap::from([
            (wgpu::TextureFormat::Rgba8Unorm, true),
            (wgpu::TextureFormat::Rgba8UnormSrgb, true),
        ]);
        assert!(available_texture_formats.contains_key(&color_format));
        let texture_extent = wgpu::Extent3d {
            width: sieze.width,
            height: sieze.height,
            depth_or_array_layers: 1,
        };

        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: color_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_texture = DepthTexture::new(sieze.width, sieze.height, device);

        FrameBuffer {
            color_texture,
            depth_texture,
        }
    }

    pub fn get_depth_texture_view(&self) -> wgpu::TextureView {
        self.depth_texture.get_view()
    }

    pub fn get_color_texture_view(&self) -> wgpu::TextureView {
        self.color_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn capture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<image::DynamicImage> {
        let texture = &self.color_texture;
        let width = self.color_texture.width();
        let height = self.color_texture.height();
        let color_type = image::ColorType::Rgba8;
        let buffer =
            crate::util::map_texture_cpu_sync(device, queue, texture, width, height, color_type);
        match image::RgbaImage::from_vec(width, height, buffer) {
            Some(image) => Some(image::DynamicImage::ImageRgba8(image)),
            None => None,
        }
    }
}
