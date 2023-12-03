use crate::shader::shader_library::ShaderLibrary;
use wgpu::{StorageTextureAccess, TextureFormat, TextureSampleType, TextureViewDimension};

const PREFIX: &str = "PanoramaToCubePipeline ";

pub struct PanoramaToCubePipeline {
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
}

impl PanoramaToCubePipeline {
    pub fn new(device: &wgpu::Device) -> PanoramaToCubePipeline {
        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("panorama_to_cube.wgsl");
        let target_format = TextureFormat::Rg11b10Float;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{PREFIX} bind_group_layout")),
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
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{PREFIX} pipeline_layout")),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("{PREFIX} compute_pipeline")),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cs_main",
        });
        PanoramaToCubePipeline {
            compute_pipeline,
            bind_group_layout,
            target_format,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        equirectangular_texture: &wgpu::Texture,
        length: u32,
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

        let cube_map_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{PREFIX} cube_map_texture")),
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
        let cube_map_texture_view = cube_map_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{PREFIX} cube_map_texture_view")),
            format: Some(cube_map_texture.format()),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

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
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(length / 16, length / 16, 6);
        }

        let _ = queue.submit(Some(encoder.finish()));
        cube_map_texture
    }
}
