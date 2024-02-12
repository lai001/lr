use wgpu::{ImageCopyTexture, Origin3d, TextureAspect, TextureFormat::*};

pub type TextureArrayData = Vec<Vec<u8>>;
pub type MipmapTextureArrayData = Vec<TextureArrayData>;

fn get_bytes_per_pixel(format: wgpu::TextureFormat) -> Option<u32> {
    if let Some(bits) = get_bits_per_pixel(format) {
        return Some(bits / 8);
    } else {
        return None;
    }
}

fn get_bits_per_pixel(format: wgpu::TextureFormat) -> Option<u32> {
    match format {
        R8Unorm => Some(8),
        R8Snorm => Some(8),
        R8Uint => Some(8),
        R8Sint => Some(8),
        R16Uint => Some(16),
        R16Sint => Some(16),
        R16Unorm => Some(16),
        R16Snorm => Some(16),
        R16Float => Some(16),
        Rg8Unorm => Some(8 * 2),
        Rg8Snorm => Some(8 * 2),
        Rg8Uint => Some(8 * 2),
        Rg8Sint => Some(8 * 2),
        R32Uint => Some(32),
        R32Sint => Some(32),
        R32Float => Some(32),
        Rg16Uint => Some(16 * 2),
        Rg16Sint => Some(16 * 2),
        Rg16Unorm => Some(16 * 2),
        Rg16Snorm => Some(16 * 2),
        Rg16Float => Some(16 * 2),
        Rgba8Unorm => Some(8 * 4),
        Rgba8UnormSrgb => Some(8 * 4),
        Rgba8Snorm => Some(8 * 4),
        Rgba8Uint => Some(8 * 4),
        Rgba8Sint => Some(8 * 4),
        Bgra8Unorm => Some(8 * 4),
        Bgra8UnormSrgb => Some(8 * 4),
        Rgb9e5Ufloat => Some(32),
        Rgb10a2Uint => Some(32),
        Rgb10a2Unorm => Some(32),
        Rg11b10Float => Some(32),
        Rg32Uint => Some(32 * 2),
        Rg32Sint => Some(32 * 2),
        Rg32Float => Some(32 * 2),
        Rgba16Uint => Some(16 * 4),
        Rgba16Sint => Some(16 * 4),
        Rgba16Unorm => Some(16 * 4),
        Rgba16Snorm => Some(16 * 4),
        Rgba16Float => Some(16 * 4),
        Rgba32Uint => Some(32 * 4),
        Rgba32Sint => Some(32 * 4),
        Rgba32Float => Some(32 * 4),
        Stencil8 => Some(8),
        Depth16Unorm => Some(16),
        Depth32Float => Some(32),
        _ => None,
    }
}

pub fn map_texture_full(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> crate::error::Result<MipmapTextureArrayData> {
    let mut datas: MipmapTextureArrayData = vec![];
    let mip_level_count = texture.mip_level_count();
    for mip_level in 0..mip_level_count {
        match map_texture_options(device, queue, texture, Some(mip_level), None) {
            Ok(texture_array_data) => {
                datas.push(texture_array_data);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    return Ok(datas);
}

pub fn map_texture_options(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    mip_level: Option<u32>,
    depth_or_array_layers: Option<u32>,
) -> crate::error::Result<TextureArrayData> {
    if texture.format().is_compressed() {
        return Err(crate::error::Error::Sync(Some(
            "Only support uncompressed texture format.".to_string(),
        )));
    }
    let Some(bytes_per_pixel) = get_bytes_per_pixel(texture.format()) else {
        return Err(crate::error::Error::Sync(Some(format!(
            "Not support texture format {:?}.",
            texture.format()
        ))));
    };

    let mip_level_size = texture
        .size()
        .mip_level_size(mip_level.unwrap_or(0), texture.dimension());
    let buffer_dimensions = crate::buffer_dimensions::BufferDimensions::new(
        mip_level_size.width as usize,
        mip_level_size.height as usize,
        bytes_per_pixel as usize,
    );
    let depth_or_array_layers =
        depth_or_array_layers.unwrap_or(texture.size().depth_or_array_layers);
    let depth_or_array_layers = depth_or_array_layers.min(texture.size().depth_or_array_layers);
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (buffer_dimensions.padded_bytes_per_row
            * buffer_dimensions.height
            * depth_or_array_layers as usize) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let copy_size = wgpu::Extent3d {
        width: buffer_dimensions.width as u32,
        height: buffer_dimensions.height as u32,
        depth_or_array_layers,
    };
    let source_image_copy_texture = ImageCopyTexture {
        texture,
        mip_level: mip_level.unwrap_or(0),
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
    };
    let destination_image_copy_buffer = wgpu::ImageCopyBuffer {
        buffer: &output_buffer,
        layout: wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
            rows_per_image: None,
        },
    };
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Read"),
    });

    encoder.copy_texture_to_buffer(
        source_image_copy_texture,
        destination_image_copy_buffer,
        copy_size,
    );
    let command_buffer = encoder.finish();
    let submission_index = queue.submit(std::iter::once(command_buffer));
    let single_image_buf_length =
        buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height as usize;
    let buffer_slice = output_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
    device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
    match receiver.recv() {
        Ok(result) => match result {
            Ok(_) => {
                let mut image_datas: Vec<Vec<u8>> = vec![];
                let padded_buffer = buffer_slice.get_mapped_range();
                let mut chunk = padded_buffer.chunks_exact(single_image_buf_length);
                while let Some(single_image) = chunk.next() {
                    image_datas.push(single_image.to_vec());
                }
                drop(padded_buffer);
                output_buffer.unmap();
                return Ok(image_datas);
            }
            Err(err) => {
                return Err(crate::error::Error::Sync(Some(err.to_string())));
            }
        },
        Err(err) => {
            return Err(crate::error::Error::Sync(Some(err.to_string())));
        }
    }
}
