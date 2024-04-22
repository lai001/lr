use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    global_shaders::grid::GridShader,
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::MeshVertex0,
    VertexBufferType,
};
use std::sync::Arc;
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    pub model: glam::Mat4,
}

pub struct GridPipeline {
    base_render_pipeline: Arc<BaseRenderPipeline>,
}

impl GridPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> GridPipeline {
        let builder = BaseRenderPipelineBuilder::default()
            .set_shader(&GridShader {})
            .set_targets(vec![Some(ColorTargetState {
                format: texture_format.clone(),
                blend: Some(BlendState {
                    color: BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: BlendComponent::OVER,
                }),
                write_mask: ColorWrites::ALL,
            })])
            .set_depth_stencil(Some(DepthStencilState {
                depth_compare: CompareFunction::Less,
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }))
            .set_vertex_buffer_type(Some(VertexBufferType::Interleaved(vec![
                MeshVertex0::type_layout(),
            ])))
            .set_primitive(Some(PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                ..Default::default()
            }));
        let base_render_pipeline = pool.get(device, shader_library, &GridShader {}, &builder);

        GridPipeline {
            base_render_pipeline,
        }
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
        self.base_render_pipeline.draw_resources2(
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
        );
    }
}
