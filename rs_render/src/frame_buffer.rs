use std::collections::HashSet;

use crate::depth_texture::DepthTexture;
use wgpu::TextureFormat;

pub struct FrameBuffer {
    color_texture: wgpu::Texture,
    depth_texture: Option<DepthTexture>,
}

impl FrameBuffer {
    pub fn new(
        device: &wgpu::Device,
        size: glam::UVec2,
        color_format: TextureFormat,
        depth_texture: Option<DepthTexture>,
        label: Option<&str>,
    ) -> FrameBuffer {
        let available_texture_formats = HashSet::from([
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba32Uint,
        ]);
        assert!(available_texture_formats.contains(&color_format));
        let texture_extent = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };
        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
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

        FrameBuffer {
            color_texture,
            depth_texture,
        }
    }

    pub fn get_depth_texture_view(&self) -> Option<wgpu::TextureView> {
        match &self.depth_texture {
            Some(depth_texture) => Some(depth_texture.get_view()),
            None => None,
        }
    }

    pub fn get_color_texture_view(&self) -> wgpu::TextureView {
        self.color_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn get_color_texture(&self) -> &wgpu::Texture {
        &self.color_texture
    }
}
