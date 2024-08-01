use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    global_shaders::{global_shader::GlobalShader, primitive::PrimitiveShader},
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::MeshVertex3,
    VertexBufferType,
};
use std::sync::Arc;
use type_layout::TypeLayout;
use wgpu::*;

pub struct PrimitiveRenderPipeline {
    pub base_render_pipeline: Arc<BaseRenderPipeline>,
    _builder: BaseRenderPipelineBuilder,
}

impl PrimitiveRenderPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> crate::error::Result<PrimitiveRenderPipeline> {
        let mut builder = BaseRenderPipelineBuilder::default();
        builder.targets = vec![Some(ColorTargetState {
            format: texture_format.clone(),
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        builder.shader_name = PrimitiveShader {}.get_name();
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Less,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        builder.vertex_buffer_type = Some(VertexBufferType::Interleaved(vec![
            MeshVertex3::type_layout(),
        ]));

        builder.primitive = Some(PrimitiveState {
            topology: PrimitiveTopology::LineList,
            cull_mode: None,
            polygon_mode: PolygonMode::Line,
            ..Default::default()
        });

        let base_render_pipeline = pool.get(device, shader_library, &builder);

        Ok(PrimitiveRenderPipeline {
            base_render_pipeline,
            _builder: builder,
        })
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        mesh_buffers: &[GpuVertexBufferImp],
        binding_resource: Vec<Vec<BindingResource<'_>>>,
    ) {
        self.base_render_pipeline.draw_resources(
            device,
            queue,
            binding_resource,
            mesh_buffers,
            &[ColorAttachment {
                color_ops: None,
                view: output_view,
                resolve_target: None,
            }],
            None,
            None,
            Some(depth_view),
            None,
            None,
        );
    }
}
