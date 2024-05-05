use std::sync::Arc;
use wgpu::*;

pub struct DefaultTextures {
    black: Arc<Texture>,
    white: Arc<Texture>,
    normal_texture: Arc<Texture>,
    black_u32: Arc<Texture>,
}

impl DefaultTextures {
    pub fn new(device: &Device, queue: &Queue) -> DefaultTextures {
        let black = Arc::new(Self::create_pure_color_rgba8_texture(
            device,
            queue,
            4,
            4,
            &Color::BLACK,
            Some("DefaultTextures.Black"),
        ));
        let white = Arc::new(Self::create_pure_color_rgba8_texture(
            device,
            queue,
            4,
            4,
            &Color::WHITE,
            Some("DefaultTextures.White"),
        ));
        let normal_texture = Arc::new(Self::create_pure_color_rgba8_texture(
            device,
            queue,
            4,
            4,
            &Color {
                r: 0.5,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
            Some("DefaultTextures.Normal"),
        ));

        let black_u32 = Arc::new(device.create_texture(&TextureDescriptor {
            label: Some("DefaultTextures.Black"),
            size: Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rg32Uint,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        DefaultTextures {
            black,
            white,
            normal_texture,
            black_u32,
        }
    }

    fn texture2d_from_rgba32f_image(
        device: &Device,
        queue: &Queue,
        image: &image::Rgba32FImage,
        label: Option<&str>,
    ) -> Texture {
        let texture_extent = Extent3d {
            depth_or_array_layers: 1,
            width: image.width(),
            height: image.height(),
        };

        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            rs_foundation::cast_to_raw_buffer(image.as_flat_samples().samples),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: None,
            },
            texture_extent,
        );
        texture
    }

    fn texture2d_from_rgba_image(
        device: &Device,
        queue: &Queue,
        image: &image::RgbaImage,
        label: Option<&str>,
    ) -> Texture {
        let texture_extent = Extent3d {
            depth_or_array_layers: 1,
            width: image.width(),
            height: image.height(),
        };

        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            image,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: None,
            },
            texture_extent,
        );
        texture
    }

    fn create_pure_color_rgba8_image(
        width: u32,
        height: u32,
        color: &Color,
    ) -> image::DynamicImage {
        let mut image = image::DynamicImage::new_rgba8(width, height);
        {
            let image = image.as_mut_rgba8().unwrap();
            for pixel in image.pixels_mut() {
                let pixel = &mut pixel.0;
                pixel[0] = (color.r * 255.0).clamp(0.0, 255.0) as u8;
                pixel[1] = (color.g * 255.0).clamp(0.0, 255.0) as u8;
                pixel[2] = (color.b * 255.0).clamp(0.0, 255.0) as u8;
                pixel[3] = (color.a * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
        image
    }

    fn create_pure_color_rgbaf32_image(
        width: u32,
        height: u32,
        color: &Color,
    ) -> image::DynamicImage {
        let mut image = image::DynamicImage::new_rgba32f(width, height);
        {
            let image = image.as_mut_rgba32f().unwrap();
            for pixel in image.pixels_mut() {
                let pixel = &mut pixel.0;
                pixel[0] = color.r.clamp(0.0, 1.0) as f32;
                pixel[1] = color.g.clamp(0.0, 1.0) as f32;
                pixel[2] = color.b.clamp(0.0, 1.0) as f32;
                pixel[3] = color.a.clamp(0.0, 1.0) as f32;
            }
        }
        image
    }

    fn create_pure_color_rgbaf32_texture(
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        color: &Color,
        label: Option<&str>,
    ) -> Texture {
        let image = Self::create_pure_color_rgbaf32_image(width, height, color);
        Self::texture2d_from_rgba32f_image(device, queue, image.as_rgba32f().unwrap(), label)
    }

    fn create_pure_color_rgba8_texture(
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        color: &Color,
        label: Option<&str>,
    ) -> Texture {
        let image = Self::create_pure_color_rgba8_image(width, height, color);
        Self::texture2d_from_rgba_image(device, queue, image.as_rgba8().unwrap(), label)
    }

    pub fn get_black_texture(&self) -> Arc<Texture> {
        self.black.clone()
    }

    pub fn get_white_texture(&self) -> Arc<Texture> {
        self.white.clone()
    }

    pub fn get_normal_texture(&self) -> Arc<Texture> {
        self.normal_texture.clone()
    }

    pub fn get_black_texture_view(&self) -> TextureView {
        self.black.create_view(&TextureViewDescriptor::default())
    }

    pub fn get_white_texture_view(&self) -> TextureView {
        self.white.create_view(&TextureViewDescriptor::default())
    }

    pub fn get_normal_texture_view(&self) -> TextureView {
        self.normal_texture
            .create_view(&TextureViewDescriptor::default())
    }

    pub fn get_black_u32_texture_view(&self) -> TextureView {
        self.black_u32
            .create_view(&TextureViewDescriptor::default())
    }
}
