use crate::{
    base_render_pipeline::BaseRenderPipeline,
    global_shaders::virtual_texture_clean::VirtualTextureCleanShader,
    gpu_vertex_buffer::GpuVertexBufferImp, shader_library::ShaderLibrary,
};
use wgpu::*;

pub struct VirtualTextureFeedBackClearPipeline {
    base_render_pipeline: BaseRenderPipeline,
}

impl VirtualTextureFeedBackClearPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
    ) -> VirtualTextureFeedBackClearPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            shader_library,
            &VirtualTextureCleanShader {},
            &[Some(texture_format.clone().into())],
            Some(DepthStencilState {
                depth_compare: CompareFunction::Never,
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            None,
            None,
            Some(PrimitiveState {
                cull_mode: None,
                ..Default::default()
            }),
            None,
            None,
        );

        VirtualTextureFeedBackClearPipeline {
            base_render_pipeline,
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
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
            Some(Operations {
                load: LoadOp::Clear(Color::TRANSPARENT),
                store: StoreOp::Store,
            }),
            Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            Some(Operations {
                load: LoadOp::Clear(0),
                store: StoreOp::Store,
            }),
            output_view,
            None,
            Some(depth_view),
        );
    }
}