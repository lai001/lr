use crate::shader::shader_library::ShaderLibrary;
use wgpu::*;

pub struct UpdatePageTableCSPipeline {
    compute_pipeline: ComputePipeline,
    textures_bind_group_layout: BindGroupLayout,
}

impl UpdatePageTableCSPipeline {
    pub fn new(device: &wgpu::Device) -> UpdatePageTableCSPipeline {
        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("update_page_table.cs.wgsl");
        let textures_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("UpdatePageTableCSPipeline"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::Rgba16Uint,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba8Uint,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&textures_bind_group_layout],
            push_constant_ranges: &[],
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cs_main",
        });
        UpdatePageTableCSPipeline {
            compute_pipeline,
            textures_bind_group_layout,
        }
    }

    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        feed_back_texture: &wgpu::Texture,
        page_table: &wgpu::Texture,
    ) {
        debug_assert_eq!(feed_back_texture.format(), TextureFormat::Rgba16Uint);
        debug_assert_eq!(page_table.format(), TextureFormat::Rgba8Uint);
        let feed_back_texture_view =
            feed_back_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let page_table_texture_view =
            page_table.create_view(&wgpu::TextureViewDescriptor::default());
        let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.textures_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&feed_back_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&page_table_texture_view),
                },
            ],
        });

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &textures_bind_group, &[]);
            cpass.dispatch_workgroups(
                feed_back_texture.width() / 16,
                feed_back_texture.height() / 16,
                6,
            );
        }
        let _ = queue.submit(Some(encoder.finish()));
    }
}
