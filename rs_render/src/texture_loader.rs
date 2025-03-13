use image::{imageops, ImageError};
use std::path::Path;
use wgpu::{util::DeviceExt, *};

use crate::misc::find_most_compatible_texture_usages;

pub struct TextureLoader {}

impl TextureLoader {
    pub fn load_texture_2d_from_file(
        file_path: &Path,
        label: Option<&str>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: Option<TextureFormat>,
        sample_count: Option<u32>,
        mipmap: Option<u8>,
        usage: Option<TextureUsages>,
    ) -> Result<Texture, ImageError> {
        let format = format.unwrap_or(TextureFormat::Rgba8Unorm);
        let usage = usage.unwrap_or(find_most_compatible_texture_usages(format));
        assert!(crate::misc::is_compatible(format, usage));
        let mipmap = mipmap.unwrap_or(1);
        let sample_count = sample_count.unwrap_or(1);

        let images = Self::make_images(file_path, mipmap, None)?;
        let texture_extent = wgpu::Extent3d {
            width: images[0].width(),
            height: images[0].height(),
            depth_or_array_layers: 1,
        };

        let texture_descriptor = TextureDescriptor {
            label,
            size: texture_extent,
            mip_level_count: images.len() as u32,
            sample_count,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        };
        let mut data: Vec<u8> = vec![];

        for image in &images {
            let raw_buffer = image.as_raw();
            data.append(&mut raw_buffer.clone());
        }

        let texture = device.create_texture_with_data(
            queue,
            &texture_descriptor,
            util::TextureDataOrder::default(),
            &data,
        );

        Ok(texture)
    }

    fn make_images(
        file_path: &Path,
        mipmap: u8,
        filter: Option<imageops::FilterType>,
    ) -> Result<Vec<image::RgbaImage>, ImageError> {
        let image = image::open(file_path)?;
        let max_mipmap = (image.width().min(image.height()).ilog2() + 1).min(mipmap as u32);
        let mut images: Vec<image::RgbaImage> = vec![];
        let texture_extent = wgpu::Extent3d {
            depth_or_array_layers: 1,
            width: image.width(),
            height: image.height(),
        };

        for mip in 0..max_mipmap {
            let extent = texture_extent.mip_level_size(mip, TextureDimension::D2);
            let image = image
                .resize(
                    extent.width,
                    extent.height,
                    filter.unwrap_or(imageops::FilterType::Nearest),
                )
                .to_rgba8();
            images.push(image);
        }

        Ok(images)
    }
}
