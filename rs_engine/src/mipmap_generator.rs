use crate::error::Result;
use image::{imageops, DynamicImage};
use rs_render::buffer_dimensions::BufferDimensions;
use std::path::Path;
use wgpu::*;

pub struct MipmapGenerator {}

impl MipmapGenerator {
    pub fn generate_from_file_cpu<P: AsRef<Path>>(
        file_path: P,
        in_max_level: Option<u32>,
        filter: Option<imageops::FilterType>,
    ) -> Result<Vec<DynamicImage>> {
        let dynamic_image =
            image::open(file_path).map_err(|err| crate::error::Error::ImageError(err, None))?;

        Ok(Self::generate_from_image_cpu(
            dynamic_image,
            in_max_level,
            filter,
        ))
    }

    pub fn generate_from_image_cpu(
        dynamic_image: DynamicImage,
        in_max_level: Option<u32>,
        filter: Option<imageops::FilterType>,
    ) -> Vec<DynamicImage> {
        let max_level = rs_core_minimal::misc::calculate_max_mips(
            dynamic_image.width().min(dynamic_image.height()),
        )
        .min(in_max_level.unwrap_or(u32::MAX));
        let texture_extent = wgpu::Extent3d {
            width: dynamic_image.width(),
            height: dynamic_image.height(),
            depth_or_array_layers: 1,
        };
        let mut images = vec![dynamic_image];

        for i in 1..max_level {
            let Some(last_image) = images.last() else {
                panic!()
            };

            let level_size = texture_extent.mip_level_size(i, TextureDimension::D2);
            let mipmap_image = last_image.resize(
                level_size.width,
                level_size.height,
                filter.unwrap_or(imageops::FilterType::Nearest),
            );
            images.push(mipmap_image);
        }
        images
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
