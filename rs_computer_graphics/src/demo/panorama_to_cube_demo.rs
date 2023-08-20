use crate::{
    file_manager::FileManager, shader::shader_library::ShaderLibrary, thread_pool::ThreadPool,
};
use rs_foundation::{cast_to_raw_buffer, next_highest_power_of_two};
use wgpu::{
    ImageCopyBuffer, ImageDataLayout, Origin3d, StorageTextureAccess, TextureAspect, TextureFormat,
    TextureSampleType, TextureViewDimension,
};

pub struct PanoramaToCubeDemo {
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    equirectangular_texture: wgpu::Texture,
    cube_map_texture: wgpu::Texture,
    cube_length: u32,
}

impl PanoramaToCubeDemo {
    fn image_data() -> (Vec<f32>, (u32, u32)) {
        let file_path = FileManager::default()
            .lock()
            .unwrap()
            .get_resource_path("Panorama.exr");

        match image::open(&file_path) {
            Ok(dynamic_image) => {
                if let image::DynamicImage::ImageRgba32F(dynamic_image) = dynamic_image {
                    let buffer = dynamic_image.as_flat_samples();
                    return (
                        buffer.to_vec().as_mut_slice().to_vec(),
                        dynamic_image.dimensions(),
                    );
                } else if let image::DynamicImage::ImageRgb32F(_) = dynamic_image {
                    let dynamic_image = &dynamic_image.into_rgba32f();
                    let buffer = dynamic_image.as_flat_samples();
                    return (
                        buffer.to_vec().as_mut_slice().to_vec(),
                        dynamic_image.dimensions(),
                    );
                } else {
                    panic!();
                }
            }
            Err(error) => panic!("{}", error),
        }
    }

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> PanoramaToCubeDemo {
        let (equirectangular_data, equirectangular_size) = Self::image_data();
        let mut length = equirectangular_size.0.min(equirectangular_size.1);
        length = (length as f32).round() as u32;
        length = next_highest_power_of_two(length as isize) as u32;

        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("panorama_to_cube.wgsl");

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cs_main",
        });

        let equirectangular_texture_extent3d = wgpu::Extent3d {
            width: equirectangular_size.0,
            height: equirectangular_size.1,
            depth_or_array_layers: 1,
        };
        let equirectangular_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: equirectangular_texture_extent3d,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let cube_map_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: length,
                height: length,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        queue.write_texture(
            equirectangular_texture.as_image_copy(),
            &cast_to_raw_buffer(&equirectangular_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(equirectangular_size.0 * 4 * 4),
                rows_per_image: None,
            },
            equirectangular_texture_extent3d,
        );

        PanoramaToCubeDemo {
            cube_length: length,
            compute_pipeline,
            bind_group_layout,
            equirectangular_texture,
            cube_map_texture,
        }
    }

    pub fn execute(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let equirectangular_texture_view_desc = wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        };
        let cube_map_texture_view_desc = wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba32Float),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        };
        let equirectangular_texture_view = self
            .equirectangular_texture
            .create_view(&equirectangular_texture_view_desc);
        let cube_map_texture_view = self
            .cube_map_texture
            .create_view(&cube_map_texture_view_desc);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&equirectangular_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cube_map_texture_view),
                },
            ],
        });

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(self.cube_length / 16, self.cube_length / 16, 6);
        }

        let copy_texutre = wgpu::ImageCopyTexture {
            texture: &self.cube_map_texture,
            mip_level: 0,
            origin: Origin3d { x: 0, y: 0, z: 0 },
            aspect: TextureAspect::All,
        };
        let size = (self.cube_length
            * self.cube_length
            * 6 as u32
            * 4 as u32
            * std::mem::size_of::<f32>() as u32) as wgpu::BufferAddress;
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let image_copy_buffer = ImageCopyBuffer {
            buffer: &staging_buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.cube_length * 4 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(self.cube_length),
            },
        };
        encoder.copy_texture_to_buffer(
            copy_texutre,
            image_copy_buffer,
            wgpu::Extent3d {
                width: self.cube_length,
                height: self.cube_length,
                depth_or_array_layers: 6,
            },
        );

        let submission_index = queue.submit(Some(encoder.finish()));
        let single_length =
            (self.cube_length * self.cube_length) as usize * 4 * std::mem::size_of::<f32>();
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
        if let Ok(Ok(_)) = receiver.recv() {
            let data = buffer_slice.get_mapped_range();
            let mut chunk = data.chunks_exact(single_length);
            let mut index: i32 = 0;

            while let Some(data) = chunk.next() {
                let deep_copy_data = data.to_vec();
                let length = self.cube_length;
                ThreadPool::io().lock().unwrap().spawn(move || {
                    match image::save_buffer(
                        std::format!("./outputimage_{}.exr", index),
                        &deep_copy_data,
                        length,
                        length,
                        image::ColorType::Rgba32F,
                    ) {
                        Ok(_) => log::debug!("Save image successfully"),
                        Err(error) => panic!("{}", error),
                    }
                });
                index += 1;
            }

            drop(data);
            staging_buffer.unmap();
        } else {
            panic!()
        }
    }
}
