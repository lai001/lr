use crate::shader::shader_library::ShaderLibrary;
use wgpu::{StorageTextureAccess, TextureFormat, TextureSampleType, TextureViewDimension};

pub struct PanoramaToCubePipeline {
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl PanoramaToCubePipeline {
    pub fn new(device: &wgpu::Device) -> PanoramaToCubePipeline {
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
        PanoramaToCubePipeline {
            compute_pipeline,
            bind_group_layout,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        equirectangular_texture: &wgpu::Texture,
        length: u32,
    ) -> wgpu::Texture {
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
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

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
        let equirectangular_texture_view =
            equirectangular_texture.create_view(&equirectangular_texture_view_desc);
        let cube_map_texture_view = cube_map_texture.create_view(&cube_map_texture_view_desc);

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
            cpass.dispatch_workgroups(length / 16, length / 16, 6);
        }

        let _ = queue.submit(Some(encoder.finish()));
        cube_map_texture
        // let image_datas = crate::util::map_texture_cube_cpu_sync(
        //     device,
        //     queue,
        //     &cube_map_texture,
        //     length,
        //     length,
        //     image::ColorType::Rgba32F,
        // );
        // let mut images: Vec<image::ImageBuffer<image::Rgba<f32>, Vec<f32>>> = vec![];
        // for image_data in &image_datas {
        //     let f32_data: &[f32] = crate::util::cast_to_type_buffer(image_data);
        //     let imgae = image::Rgba32FImage::from_vec(length, length, f32_data.to_vec()).unwrap();
        //     images.push(imgae);
        // }
        // images
    }
}