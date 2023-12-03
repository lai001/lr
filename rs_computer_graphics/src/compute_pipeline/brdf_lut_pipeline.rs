use crate::shader::shader_library::ShaderLibrary;
use wgpu::{
    BindGroupLayout, ComputePipeline, StorageTextureAccess, TextureFormat, TextureViewDimension,
};

const PREFIX: &str = "BrdfLutPipeline ";

struct Constants {
    sample_count: u32,
}

pub struct BrdfLutPipeline {
    compute_pipeline: ComputePipeline,
    textures_bind_group_layout: BindGroupLayout,
    constants_bind_group_layout: BindGroupLayout,
    target_format: wgpu::TextureFormat,
}

impl BrdfLutPipeline {
    pub fn new(device: &wgpu::Device) -> BrdfLutPipeline {
        let target_format = TextureFormat::Rg16Float;
        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("brdf_lut.wgsl");
        let textures_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{PREFIX} textures_bind_group_layout")),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: target_format,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });
        let constants_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{PREFIX} constants_bind_group_layout")),
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
            label: Some(&format!("{PREFIX} pipeline_layout")),
            bind_group_layouts: &[&textures_bind_group_layout, &constants_bind_group_layout],
            push_constant_ranges: &[],
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("{PREFIX} compute_pipeline")),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cs_main",
        });
        BrdfLutPipeline {
            compute_pipeline,
            textures_bind_group_layout,
            constants_bind_group_layout,
            target_format,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        length: u32,
        sample_count: u32,
    ) -> wgpu::Texture {
        let lut_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{PREFIX} brdf_lut_texture")),
            size: wgpu::Extent3d {
                width: length,
                height: length,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.target_format,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let lut_texture_view = lut_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{PREFIX} brdf_lut_texture_view")),
            format: Some(lut_texture.format()),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{PREFIX} textures_bind_group")),
            layout: &self.textures_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&lut_texture_view),
            }],
        });

        let constants = Constants { sample_count };

        let uniform_buf = crate::util::create_gpu_uniform_buffer_from(device, &constants, None);

        let constants_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{PREFIX} constants_bind_group")),
            layout: &self.constants_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("{PREFIX} command_encoder")),
        });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&format!("{PREFIX} compute_pass")),
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &textures_bind_group, &[]);
            compute_pass.set_bind_group(1, &constants_bind_group, &[]);
            compute_pass.dispatch_workgroups(length / 16, length / 16, 1);
        }
        let _ = queue.submit(Some(encoder.finish()));
        lut_texture
    }
}
