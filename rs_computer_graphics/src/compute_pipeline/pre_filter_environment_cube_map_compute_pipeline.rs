use crate::shader::shader_library::ShaderLibrary;
use wgpu::{
    BindGroupLayout, ComputePipeline, StorageTextureAccess, TextureSampleType, TextureViewDimension,
};

const PREFIX: &str = "PreFilterEnvironmentCubeMapComputePipeline ";

struct Constants {
    roughness: f32,
    sample_count: u32,
}

pub struct PreFilterEnvironmentCubeMapComputePipeline {
    compute_pipeline: ComputePipeline,
    textures_bind_group_layout: BindGroupLayout,
    constants_bind_group_layout: BindGroupLayout,
    target_format: wgpu::TextureFormat,
}

impl PreFilterEnvironmentCubeMapComputePipeline {
    pub fn new(device: &wgpu::Device) -> PreFilterEnvironmentCubeMapComputePipeline {
        let target_format = wgpu::TextureFormat::Rg11b10Float;
        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("pre_filter_environment_cube_map.wgsl");
        let textures_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{PREFIX} textures_bind_group_layout")),

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
                            format: target_format,
                            view_dimension: TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                ],
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
        PreFilterEnvironmentCubeMapComputePipeline {
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
        equirectangular_texture: &wgpu::Texture,
        length: u32,
        roughness: f32,
        sample_count: u32,
    ) -> wgpu::Texture {
        let equirectangular_texture_view =
            equirectangular_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("{PREFIX} equirectangular_texture_view")),
                format: Some(equirectangular_texture.format()),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

        let prefilter_cube_map_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{PREFIX} prefilter_cube_map_texture")),
            size: wgpu::Extent3d {
                width: length,
                height: length,
                depth_or_array_layers: 6,
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

        let prefilter_cube_map_texture_view =
            prefilter_cube_map_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("{PREFIX} prefilter_cube_map_texture_view")),
                format: Some(prefilter_cube_map_texture.format()),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
        let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{PREFIX} textures_bind_group")),
            layout: &self.textures_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&equirectangular_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&prefilter_cube_map_texture_view),
                },
            ],
        });

        let constants = Constants {
            roughness,
            sample_count,
        };

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
            let a = (length / 16).max(1);
            compute_pass.dispatch_workgroups(a, a, 6);
        }
        let _ = queue.submit(Some(encoder.finish()));
        return prefilter_cube_map_texture;
    }
}
