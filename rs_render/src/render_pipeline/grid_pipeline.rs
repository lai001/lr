use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    global_shaders::{global_shader::GlobalShader, grid::GridShader},
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
    multisample_pipeline: Arc<BaseRenderPipeline>,
}

impl GridPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> GridPipeline {
        let mut builder = BaseRenderPipelineBuilder::default();
        builder.targets = vec![Some(ColorTargetState {
            format: texture_format.clone(),
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        builder.shader_name = GridShader {}.get_name();
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Less,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        builder.vertex_buffer_type = Some(VertexBufferType::Interleaved(vec![
            MeshVertex0::type_layout(),
        ]));
        builder.primitive = Some(PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        });

        let base_render_pipeline = pool.get(device, shader_library, &builder);
        builder.multisample = Some(MultisampleState {
            count: 4,
            ..Default::default()
        });
        let multisample_pipeline = pool.get(device, shader_library, &builder);

        GridPipeline {
            base_render_pipeline,
            multisample_pipeline,
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        resolve_target: Option<&TextureView>,
        depth_view: &TextureView,
        mesh_buffers: &[GpuVertexBufferImp],
        binding_resource: Vec<Vec<BindingResource<'_>>>,
    ) {
        let render_pipeline;
        let color_attachment = if let Some(resolve_target) = resolve_target {
            render_pipeline = &self.multisample_pipeline;
            ColorAttachment {
                color_ops: None,
                view: resolve_target,
                resolve_target: Some(output_view),
            }
        } else {
            render_pipeline = &self.base_render_pipeline;
            ColorAttachment {
                color_ops: None,
                view: output_view,
                resolve_target: None,
            }
        };
        render_pipeline.draw_resources(
            device,
            queue,
            binding_resource,
            mesh_buffers,
            &[color_attachment],
            None,
            None,
            Some(depth_view),
            None,
            None,
        );
    }
}
