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
        let base_render_pipeline = BaseRenderPipeline::new(device, shader_library, builder);
        AttachmentPipeline {
            base_render_pipeline,
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        clear_color: wgpu::Color,
    ) {
        self.base_render_pipeline.draw_resources2(
            device,
            queue,
            vec![],
            &vec![GpuVertexBufferImp {
                vertex_buffers: &vec![],
                vertex_count: 0,
                index_buffer: None,
                index_count: None,
            }],
            &[ColorAttachment {
                color_ops: Some(Operations {
                    load: LoadOp::Clear(clear_color),
                    store: StoreOp::Store,
                }),
                view: output_view,
                resolve_target: None,
            }],
            Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            Some(Operations {
                load: LoadOp::Clear(0),
                store: StoreOp::Store,
            }),
            Some(depth_view),
        );
    }
}
