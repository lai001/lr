use crate::{shader::shader_library::ShaderLibrary, util::map_texture_cpu_sync};
use wgpu::{
    BindGroupLayout, ComputePipeline, StorageTextureAccess, TextureFormat, TextureViewDimension,
};

struct Constants {
    sample_count: u32,
}

pub struct BrdfLutPipeline {
    compute_pipeline: ComputePipeline,
    textures_bind_group_layout: BindGroupLayout,
    constants_bind_group_layout: BindGroupLayout,
}

impl BrdfLutPipeline {
    pub fn new(device: &wgpu::Device) -> BrdfLutPipeline {
        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("brdf_lut.wgsl");
        let textures_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });
        let constants_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Constants>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&textures_bind_group_layout, &constants_bind_group_layout],
            push_constant_ranges: &[],
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cs_main",
        });
        BrdfLutPipeline {
            compute_pipeline,
            textures_bind_group_layout,
            constants_bind_group_layout,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        length: u32,
        sample_count: u32,
    ) -> image::ImageBuffer<image::Rgba<f32>, Vec<f32>> {
        let lut_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: length,
                height: length,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let lut_texture_view_desc = wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        };
        let lut_texture_view = lut_texture.create_view(&lut_texture_view_desc);
        let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.textures_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&lut_texture_view),
            }],
        });

        let constants = Constants { sample_count };

        let uniform_buf = crate::util::create_gpu_uniform_buffer_from(device, &constants, None);

        let constants_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.constants_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &textures_bind_group, &[]);
            cpass.set_bind_group(1, &constants_bind_group, &[]);
            cpass.dispatch_workgroups(length / 16, length / 16, 6);
        }
        let _ = queue.submit(Some(encoder.finish()));
        let image_data = map_texture_cpu_sync(
            device,
            queue,
            &lut_texture,
            length,
            length,
            image::ColorType::Rgba32F,
        );
        let f32_data: &[f32] = crate::util::cast_to_type_buffer(&image_data);
        image::Rgba32FImage::from_vec(length, length, f32_data.to_vec()).unwrap()
    }
}
