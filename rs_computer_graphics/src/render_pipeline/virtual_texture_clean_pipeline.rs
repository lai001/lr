use crate::shader::shader_library::ShaderLibrary;

pub struct VirtualTextureCleanPipeline {
    pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl VirtualTextureCleanPipeline {
    pub fn new(
        device: &wgpu::Device,
        output_texture_format: &wgpu::TextureFormat,
    ) -> VirtualTextureCleanPipeline {
        assert_eq!(*output_texture_format, wgpu::TextureFormat::Rgba16Uint);
        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("virtual_texture_clean.wgsl");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let depth_stencil: Option<wgpu::DepthStencilState> = Some(wgpu::DepthStencilState {
            depth_compare: wgpu::CompareFunction::GreaterEqual,
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState::from(
                    output_texture_format.clone(),
                ))],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        VirtualTextureCleanPipeline {
            pipeline_layout,
            render_pipeline,
        }
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        color_ops: wgpu::Operations<wgpu::Color>,
        depth_ops: Option<wgpu::Operations<f32>>,
        stencil_ops: Option<wgpu::Operations<u32>>,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let render_pass_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
            view: &depth_view,
            depth_ops,
            stencil_ops,
        };
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: color_ops,
                })],
                depth_stencil_attachment: Some(render_pass_depth_stencil_attachment),
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..5, 0..1);
        }
        queue.submit(std::iter::once(encoder.finish()));
    }
}
