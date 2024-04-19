use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    global_shaders::grid::GridShader,
    gpu_buffer,
    gpu_vertex_buffer::GpuVertexBufferImp,
    sampler_cache::SamplerCache,
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
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
}

pub struct GridPipeline {
    base_render_pipeline: Arc<BaseRenderPipeline>,
    builder: BaseRenderPipelineBuilder,
    sampler: Arc<Sampler>,
}

impl GridPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        sampler_cache: &mut SamplerCache,
        pool: &mut BaseRenderPipelinePool,
    ) -> GridPipeline {
        let builder = BaseRenderPipelineBuilder::default()
            .set_targets(vec![Some(texture_format.clone().into())])
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

        let sampler = sampler_cache.create_sampler(device, &SamplerDescriptor::default());

        GridPipeline {
            base_render_pipeline,
            sampler,
            builder,
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        constants: &Constants,
        mesh_buffers: &[GpuVertexBufferImp],
    ) {
        let uniform_buf =
            gpu_buffer::uniform::from(device, constants, Some("GridPipeline.constants"));

        self.base_render_pipeline.draw_resources2(
            device,
            queue,
            vec![vec![uniform_buf.as_entire_binding()]],
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
