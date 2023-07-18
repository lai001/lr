use crate::{buffer_dimensions::BufferDimensions, thread_pool};

pub struct CaptureScreen {}

impl CaptureScreen {
    pub fn capture(
        path: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        format: wgpu::TextureFormat,
        window_size: &winit::dpi::PhysicalSize<u32>,
    ) {
        let bytes_per_pixel: usize = format.block_size(None).unwrap() as usize;
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let buffer_dimensions = BufferDimensions::new(
            window_size.width as usize,
            window_size.height as usize,
            bytes_per_pixel,
        );
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_extent = wgpu::Extent3d {
            width: buffer_dimensions.width as u32,
            height: buffer_dimensions.height as u32,
            depth_or_array_layers: 1,
        };

        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
                    rows_per_image: None,
                },
            },
            texture_extent,
        );

        let command_buffer = encoder.finish();
        let submission_index = queue.submit(std::iter::once(command_buffer));

        let buffer_slice = output_buffer.slice(..);

        let (sender, receiver) = std::sync::mpsc::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));

        if let Ok(Ok(_)) = receiver.recv() {
            let padded_buffer = buffer_slice.get_mapped_range();
            let deep_copy_data = padded_buffer.to_vec();
            let path = path.to_string();
            let window_size = window_size.clone();
            thread_pool::ThreadPool::global().lock().unwrap().spawn(
                move || match image::save_buffer(
                    path,
                    &deep_copy_data,
                    window_size.width,
                    window_size.height,
                    image::ColorType::Rgba8,
                ) {
                    Ok(_) => log::debug!("Save screen image successfully"),
                    Err(error) => log::error!("{:?}", error),
                },
            );
            drop(padded_buffer);
            output_buffer.unmap();
        }
    }
}
