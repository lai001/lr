use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::BaseRenderPipelineBuilder,
    global_shaders::{attachment::AttachmentShader, global_shader::GlobalShader},
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
};
use wgpu::*;

pub struct AttachmentPipeline {
    base_render_pipeline: BaseRenderPipeline,
    clear_depth_pipeline: BaseRenderPipeline,
}

impl AttachmentPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
    ) -> AttachmentPipeline {
        let mut builder = BaseRenderPipelineBuilder::default();
        builder.shader_name = AttachmentShader {}.get_name();
        builder.targets = vec![Some(texture_format.clone().into())];
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Never,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        builder.primitive = Some(PrimitiveState {
            cull_mode: None,
            ..Default::default()
        });
        let base_render_pipeline = BaseRenderPipeline::new(device, shader_library, builder.clone());
        builder.targets = vec![];
        let clear_depth_pipeline = BaseRenderPipeline::new(device, shader_library, builder);
        AttachmentPipeline {
            base_render_pipeline,
            clear_depth_pipeline,
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: Option<(&TextureView, wgpu::Color)>,
        depth_view: Option<&TextureView>,
    ) {
        let base_render_pipeline = match (output_view, depth_view) {
            (None, None) => panic!(),
            (None, Some(_)) => &self.clear_depth_pipeline,
            (Some(_), None) => unimplemented!(),
            (Some(_), Some(_)) => &self.base_render_pipeline,
        };
        let color_attachments = if let Some((output_view, clear_color)) = output_view {
            vec![ColorAttachment {
                color_ops: Some(Operations {
                    load: LoadOp::Clear(clear_color),
                    store: StoreOp::Store,
                }),
                view: output_view,
                resolve_target: None,
            }]
        } else {
            vec![]
        };

        base_render_pipeline.draw_resources2(
            device,
            queue,
            vec![],
            &vec![GpuVertexBufferImp {
                vertex_buffers: &vec![],
                vertex_count: 0,
                index_buffer: None,
                index_count: None,
            }],
            &color_attachments,
            Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            Some(Operations {
                load: LoadOp::Clear(0),
                store: StoreOp::Store,
            }),
            depth_view,
            None,
            None,
        );
    }
}
