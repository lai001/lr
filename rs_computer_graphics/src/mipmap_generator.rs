use image::{imageops, DynamicImage, ImageBuffer, Rgba};
use wgpu::*;

use crate::buffer_dimensions::BufferDimensions;

pub struct MipmapGenerator {}

impl MipmapGenerator {
    pub fn generate_from_file_cpu(file_path: &str, in_max_level: Option<u32>) -> Vec<DynamicImage> {
        let mut images = vec![];
        match image::open(file_path) {
            Ok(dynamic_image) => {
                images.append(&mut Self::generate_from_image_cpu(
                    &dynamic_image,
                    in_max_level,
                ));
                images.insert(0, dynamic_image);
            }
            Err(error) => {
                log::warn!("{error}");
            }
        }
        return images;
    }

    pub fn generate_from_image_cpu(
        dynamic_image: &DynamicImage,
        in_max_level: Option<u32>,
    ) -> Vec<DynamicImage> {
        let mut images = vec![];
        let texture_extent = wgpu::Extent3d {
            width: dynamic_image.width(),
            height: dynamic_image.height(),
            depth_or_array_layers: 1,
        };

        let max_level: u32;
        match in_max_level {
            Some(in_max_level) => {
                max_level =
                    in_max_level.min(dynamic_image.width().min(dynamic_image.height()).ilog2() + 1)
            }
            None => {
                max_level = dynamic_image.width().min(dynamic_image.height()).ilog2() + 1;
            }
        }

        for i in 1..max_level {
            let level_size = texture_extent.mip_level_size(i, TextureDimension::D2);
            let mipmap_image = dynamic_image.resize(
                level_size.width,
                level_size.height,
                imageops::FilterType::Nearest,
            );
            images.push(mipmap_image);
        }
        return images;
    }

    pub fn generate(
        file_path: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Texture> {
        match image::open(file_path) {
            Ok(dynamic_image) => {
                let image = dynamic_image.to_rgba8();
                let texture_extent = wgpu::Extent3d {
                    depth_or_array_layers: 1,
                    width: image.width(),
                    height: image.height(),
                };
                let buffer_dimensions = BufferDimensions::new(
                    texture_extent.width as usize,
                    texture_extent.height as usize,
                    4,
                );

                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: texture_extent,
                    mip_level_count: image.width().min(image.height()).ilog2() + 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });

                queue.write_texture(
                    texture.as_image_copy(),
                    &image,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
                        rows_per_image: None,
                    },
                    texture_extent,
                );
                // TODO: Generate mipmaps with compute shader
                Some(texture)
            }
            Err(error) => {
                log::warn!("{error}");
                None
            }
        }
    }
}
