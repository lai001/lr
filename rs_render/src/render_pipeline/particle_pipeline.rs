use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    global_shaders::{global_shader::GlobalShader, particle::ParticleShader},
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::{Instance0, MeshVertex0},
    VertexBufferType,
};
use std::{collections::HashMap, sync::Arc};
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    pub model: glam::Mat4,
}

pub struct ParticlePipeline {
    base_render_pipeline: Arc<BaseRenderPipeline>,
}

impl ParticlePipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> ParticlePipeline {
        let mut builder = BaseRenderPipelineBuilder::default();
        builder.targets = vec![Some(ColorTargetState {
            format: texture_format.clone(),
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        builder.shader_name = ParticleShader {}.get_name();
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Less,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: false,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        builder.vertex_buffer_type = Some(VertexBufferType::Interleaved(vec![
            MeshVertex0::type_layout(),
            Instance0::type_layout(),
        ]));
        builder.primitive = Some(PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        });
        builder.vertex_layout_hooks = Some(HashMap::from([(1, VertexStepMode::Instance)]));

        let base_render_pipeline = pool.get(device, shader_library, &builder);

        ParticlePipeline {
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
