use crate::buffer_dimensions::BufferDimensions;
use wgpu::*;

pub struct Atlas {
    texture: Texture,
    datas: Vec<f32>,
}

impl Atlas {
    pub fn new(device: &Device, width: u32) -> Atlas {
        let texture_extent = wgpu::Extent3d {
            depth_or_array_layers: 1,
            width,
            height: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Atlas"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        Atlas {
            texture,
            datas: vec![0.0; width as usize],
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, datas: &[f32]) {
        let len = datas.len().min(self.datas.len());
        (&mut self.datas[0..len]).copy_from_slice(datas);

        let buffer_dimensions = BufferDimensions::new(
            self.texture.size().width as usize,
            self.texture.size().height as usize,
            std::mem::size_of::<f32>(),
        );
        queue.write_texture(
            self.texture.as_image_copy(),
            rs_foundation::cast_to_raw_buffer(&self.datas),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
                rows_per_image: None,
            },
            self.texture.size(),
        );
    }

    pub fn get_texture(&self) -> &Texture {
        &self.texture
    }
}
