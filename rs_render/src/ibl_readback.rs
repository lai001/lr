use crate::acceleration_bake::AccelerationBaker;
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
        if let Some(brdflut_texture) = baker.get_brdflut_texture().as_ref() {
            if brdflut_texture.format() != wgpu::TextureFormat::Rgba32Float {
                return Err(crate::error::Error::Sync(Some(format!(
                    "Not support format {:?}.",
                    brdflut_texture.format()
                ))));
            }
            let bake_info = baker.get_bake_info();
            let image_data =
                crate::texture_readback::map_texture_full(device, queue, &brdflut_texture);
            match image_data {
                Ok(image_data) => {
                    let buffer = &image_data[0][0];
                    let f32_data: &[f32] = rs_foundation::cast_to_type_buffer(&buffer);
                    let image = image::Rgba32FImage::from_vec(
                        bake_info.brdflutmap_length,
                        bake_info.brdflutmap_length,
                        f32_data.to_vec(),
                    );
                    match image {
                        Some(image) => {
                            return Ok(image::DynamicImage::ImageRgba32F(image));
                        }
                        None => {
                            return Err(crate::error::Error::Sync(Some(
                                "The container is not big enough.".to_string(),
                            )));
                        }
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        } else {
            return Err(crate::error::Error::Sync(Some(
                "Texture is null.".to_string(),
            )));
        }
    }
}
