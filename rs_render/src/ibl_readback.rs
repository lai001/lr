use crate::acceleration_bake::AccelerationBaker;
use crate::cube_map::CubeMap;
use crate::error::Result;

pub struct IBLReadBack {}

impl IBLReadBack {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_brdflut_texture(
        baker: &AccelerationBaker,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<image::DynamicImage> {
        let brdflut_texture = baker.get_brdflut_texture();
        let Some(brdflut_texture) = brdflut_texture.as_ref() else {
            return Err(crate::error::Error::Sync(Some(
                "Texture is null.".to_string(),
            )));
        };

        if brdflut_texture.format() != wgpu::TextureFormat::Rgba32Float {
            return Err(crate::error::Error::Sync(Some(format!(
                "Not support format {:?}.",
                brdflut_texture.format()
            ))));
        }
        let bake_info = baker.get_bake_info();
        let image_data =
            crate::texture_readback::map_texture_full(device, queue, &brdflut_texture)?;
        let buffer = &image_data[0][0];
        let f32_data: &[f32] = rs_foundation::cast_to_type_buffer(&buffer);
        let image = image::Rgba32FImage::from_vec(
            bake_info.brdflutmap_length,
            bake_info.brdflutmap_length,
            f32_data.to_vec(),
        )
        .ok_or(crate::error::Error::Sync(Some(
            "The container is not big enough.".to_string(),
        )))?;

        Ok(image::DynamicImage::ImageRgba32F(image))
    }

    pub fn read_irradiance_cube_map_texture(
        baker: &AccelerationBaker,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<CubeMap<image::Rgba<f32>, Vec<f32>>> {
        let irradiance_texture = baker.get_irradiance_cube_map_texture();
        let Some(irradiance_texture) = irradiance_texture.as_ref() else {
            return Err(crate::error::Error::Sync(Some(
                "Texture is null.".to_string(),
            )));
        };

        if irradiance_texture.format() != wgpu::TextureFormat::Rgba32Float {
            return Err(crate::error::Error::Sync(Some(format!(
                "Not support format {:?}.",
                irradiance_texture.format()
            ))));
        }
        Self::read_cube_map(irradiance_texture, device, queue)
    }

    fn read_cube_map(
        texture: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<CubeMap<image::Rgba<f32>, Vec<f32>>> {
        let size = texture.size().width;
        if texture.format() != wgpu::TextureFormat::Rgba32Float {
            return Err(crate::error::Error::Sync(Some(format!(
                "Not support format {:?}.",
                texture.format()
            ))));
        }
        let image_data = crate::texture_readback::map_texture_full(device, queue, texture)?;
        let cube_map = CubeMap {
            negative_x: Self::build_image_buffer(&image_data[0][0], size)?,
            positive_x: Self::build_image_buffer(&image_data[0][1], size)?,
            negative_y: Self::build_image_buffer(&image_data[0][2], size)?,
            positive_y: Self::build_image_buffer(&image_data[0][3], size)?,
            negative_z: Self::build_image_buffer(&image_data[0][4], size)?,
            positive_z: Self::build_image_buffer(&image_data[0][5], size)?,
        };
        Ok(cube_map)
    }

    pub fn read_pre_filter_cube_map_textures(
        baker: &AccelerationBaker,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<CubeMap<image::Rgba<f32>, Vec<f32>>>> {
        let pre_filter_textures = baker.get_pre_filter_cube_map_textures();
        let Some(pre_filter_textures) = pre_filter_textures.as_ref() else {
            return Err(crate::error::Error::Sync(Some(
                "Texture is null.".to_string(),
            )));
        };
        let mut cube_maps = vec![];
        for texture in pre_filter_textures {
            if texture.width() == 1 {
                break;
            }
            let cube_map = Self::read_cube_map(texture, device, queue)?;
            cube_maps.push(cube_map);
        }
        Ok(cube_maps)
    }

    fn build_image_buffer(
        data: &[u8],
        size: u32,
    ) -> Result<image::ImageBuffer<image::Rgba<f32>, Vec<f32>>> {
        let f32_data: &[f32] = rs_foundation::cast_to_type_buffer(data);
        image::Rgba32FImage::from_vec(size, size, f32_data.to_vec()).ok_or(
            crate::error::Error::Sync(Some("The container is not big enough.".to_string())),
        )
    }
}
