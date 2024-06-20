use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    global_shaders::{fxaa::FXAAShader, global_shader::GlobalShader},
    gpu_vertex_buffer::{Draw, EDrawCallType, GpuVertexBufferImp},
    shader_library::ShaderLibrary,
};
use std::sync::Arc;
use wgpu::*;

pub struct FXAAPipeline {
    base_render_pipeline: Arc<BaseRenderPipeline>,
}

impl FXAAPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> FXAAPipeline {
        let mut builder = BaseRenderPipelineBuilder::default();
        builder.targets = vec![Some(ColorTargetState {
            format: texture_format.clone(),
            blend: None,
            write_mask: ColorWrites::ALL,
        })];
        builder.shader_name = FXAAShader {}.get_name();

        builder.primitive = Some(PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        });

        let base_render_pipeline = pool.get(device, shader_library, &builder);

        FXAAPipeline {
            base_render_pipeline,
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        binding_resource: Vec<Vec<BindingResource<'_>>>,
    ) {
        self.base_render_pipeline.draw_resources(
            device,
            queue,
            binding_resource,
            &vec![GpuVertexBufferImp {
                vertex_buffers: &vec![],
                vertex_count: 6,
                index_buffer: None,
                index_count: None,
                draw_type: EDrawCallType::Draw(Draw { instances: 0..1 }),
            }],
            &[ColorAttachment {
                color_ops: None,
                view: output_view,
                resolve_target: None,
            }],
            None,
            None,
            None,
            None,
            None,
        );
    }
}
