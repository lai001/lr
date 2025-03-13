use std::sync::Arc;
use wgpu::*;

pub struct PrebakeIBL {
    _brdflut_texture: Arc<wgpu::Texture>,
    _irradiance_texture: Arc<wgpu::Texture>,
    _pre_filter_texture: Arc<wgpu::Texture>,

    brdflut_texture_view: TextureView,
    irradiance_texture_view: TextureView,
    pre_filter_cube_map_texture_view: TextureView,
}

impl PrebakeIBL {
    pub fn empty(device: &Device) -> crate::error::Result<PrebakeIBL> {
        let brdflut_texture = device.create_texture(&TextureDescriptor {
            label: Some("brdflut_texture"),
            size: Extent3d {
                depth_or_array_layers: 1,
                width: 4,
                height: 4,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let irradiance_texture = device.create_texture(&TextureDescriptor {
            label: Some("irradiance_texture"),
            size: Extent3d {
                depth_or_array_layers: 6,
                width: 4,
                height: 4,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let pre_filter_texture = device.create_texture(&TextureDescriptor {
            label: Some("pre_filter_texture"),
            size: Extent3d {
                depth_or_array_layers: 6,
                width: 4,
                height: 4,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let brdflut_texture_view = Self::create_brdflut_texture_view(&brdflut_texture);
        let irradiance_texture_view = Self::create_irradiance_texture_view(&irradiance_texture);
        let pre_filter_cube_map_texture_view =
            Self::create_pre_filter_cube_map_texture_view(&pre_filter_texture);

        Ok(PrebakeIBL {
            _brdflut_texture: Arc::new(brdflut_texture),
            _irradiance_texture: Arc::new(irradiance_texture),
            _pre_filter_texture: Arc::new(pre_filter_texture),
            brdflut_texture_view,
            irradiance_texture_view,
            pre_filter_cube_map_texture_view,
        })
    }

    pub fn from_surfaces(
        device: &Device,
        queue: &Queue,
        brdf_surface: image_dds::SurfaceRgba32Float<Vec<f32>>,
        irradiance_surface: image_dds::SurfaceRgba32Float<Vec<f32>>,
        pre_filter_surface: image_dds::SurfaceRgba32Float<Vec<f32>>,
    ) -> crate::error::Result<PrebakeIBL> {
        let brdf_data = brdf_surface
            .get(0, 0, 0)
            .ok_or(crate::error::Error::Other(None))?;
        let texture_extent = Extent3d {
            depth_or_array_layers: 1,
            width: brdf_surface.width,
            height: brdf_surface.height,
        };
        let brdflut_texture = device.create_texture(&TextureDescriptor {
            label: Some("brdflut_texture"),
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
            brdflut_texture.as_image_copy(),
            rs_foundation::cast_to_raw_buffer(brdf_data),
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4 * brdf_surface.width),
                rows_per_image: None,
            },
            texture_extent,
        );

        let irradiance_data = &irradiance_surface.data;
        let texture_extent = Extent3d {
            depth_or_array_layers: 6,
            width: irradiance_surface.width,
            height: irradiance_surface.height,
        };
        let irradiance_texture = device.create_texture(&TextureDescriptor {
            label: Some("irradiance_texture"),
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
            irradiance_texture.as_image_copy(),
            rs_foundation::cast_to_raw_buffer(irradiance_data),
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4 * irradiance_surface.width),
                rows_per_image: Some(irradiance_surface.height),
            },
            texture_extent,
        );

        let mut pre_filter_data: Vec<f32> = Vec::with_capacity(pre_filter_surface.data.len());
        for mipmap in 0..pre_filter_surface.mipmaps {
            for layer in 0..pre_filter_surface.layers {
                let sub_data = pre_filter_surface.get(layer, 0, mipmap).unwrap();
                pre_filter_data.append(&mut sub_data.to_vec());
            }
        }
        let pre_filter_texture = device.create_texture(&TextureDescriptor {
            label: Some("pre_filter_texture"),
            size: texture_extent,
            mip_level_count: pre_filter_surface.mipmaps,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        for mipmap in 0..pre_filter_surface.mipmaps {
            let size = rs_core_minimal::misc::get_mip_level_size(pre_filter_surface.width, mipmap);

            let texture_extent = Extent3d {
                depth_or_array_layers: 6,
                width: size,
                height: size,
            };

            let offset: usize = (0..mipmap).fold(0, |acc, m| {
                acc + pre_filter_surface.get(0, 0, m).unwrap().len()
                    * pre_filter_surface.layers as usize
            });

            queue.write_texture(
                TexelCopyTextureInfo {
                    texture: &pre_filter_texture,
                    mip_level: mipmap,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                rs_foundation::cast_to_raw_buffer(&pre_filter_data[offset..]),
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * 4 * size),
                    rows_per_image: Some(size),
                },
                texture_extent,
            );
        }

        let brdflut_texture_view = Self::create_brdflut_texture_view(&brdflut_texture);
        let irradiance_texture_view = Self::create_irradiance_texture_view(&irradiance_texture);
        let pre_filter_cube_map_texture_view =
            Self::create_pre_filter_cube_map_texture_view(&pre_filter_texture);

        Ok(PrebakeIBL {
            _brdflut_texture: Arc::new(brdflut_texture),
            _irradiance_texture: Arc::new(irradiance_texture),
            _pre_filter_texture: Arc::new(pre_filter_texture),
            brdflut_texture_view,
            irradiance_texture_view,
            pre_filter_cube_map_texture_view,
        })
    }

    pub fn get_brdflut_texture_view(&self) -> &wgpu::TextureView {
        &self.brdflut_texture_view
        // self.brdflut_texture
        //     .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn get_irradiance_texture_view(&self) -> &wgpu::TextureView {
        &self.irradiance_texture_view
        // let mip_level_count = self.irradiance_texture.mip_level_count();
        // let array_layer_count = self.irradiance_texture.depth_or_array_layers();
        // let format = self.irradiance_texture.format();
        // return self
        //     .irradiance_texture
        //     .create_view(&wgpu::TextureViewDescriptor {
        //         label: None,
        //         format: Some(format),
        //         dimension: Some(wgpu::TextureViewDimension::Cube),
        //         aspect: wgpu::TextureAspect::All,
        //         base_mip_level: 0,
        //         mip_level_count: Some(mip_level_count),
        //         base_array_layer: 0,
        //         array_layer_count: Some(array_layer_count),
        //     });
    }

    pub fn get_pre_filter_cube_map_texture_view(&self) -> &wgpu::TextureView {
        &self.pre_filter_cube_map_texture_view
        // let mip_level_count = self.pre_filter_texture.mip_level_count();
        // let array_layer_count = self.pre_filter_texture.depth_or_array_layers();
        // let format = self.pre_filter_texture.format();
        // return self
        //     .pre_filter_texture
        //     .create_view(&wgpu::TextureViewDescriptor {
        //         label: None,
        //         format: Some(format),
        //         dimension: Some(wgpu::TextureViewDimension::Cube),
        //         aspect: wgpu::TextureAspect::All,
        //         base_mip_level: 0,
        //         mip_level_count: Some(mip_level_count),
        //         base_array_layer: 0,
        //         array_layer_count: Some(array_layer_count),
        //     });
    }

    pub fn create_brdflut_texture_view(brdflut_texture: &Texture) -> wgpu::TextureView {
        brdflut_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn create_irradiance_texture_view(irradiance_texture: &Texture) -> wgpu::TextureView {
        let mip_level_count = irradiance_texture.mip_level_count();
        let array_layer_count = irradiance_texture.depth_or_array_layers();
        let format = irradiance_texture.format();
        return irradiance_texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(mip_level_count),
            base_array_layer: 0,
            array_layer_count: Some(array_layer_count),
            usage: None,
        });
    }

    pub fn create_pre_filter_cube_map_texture_view(
        pre_filter_texture: &Texture,
    ) -> wgpu::TextureView {
        let mip_level_count = pre_filter_texture.mip_level_count();
        let array_layer_count = pre_filter_texture.depth_or_array_layers();
        let format = pre_filter_texture.format();
        return pre_filter_texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(mip_level_count),
            base_array_layer: 0,
            array_layer_count: Some(array_layer_count),
            usage: None,
        });
    }
}
