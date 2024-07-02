use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    command::{MaterialRenderPipelineHandle, Viewport},
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::{MeshVertex0, MeshVertex1, MeshVertex2},
    view_mode::EViewModeType,
    VertexBufferType,
};
use std::sync::Arc;
use type_layout::TypeLayout;
use wgpu::*;

pub struct MaterialRenderPipeline {
    pub base_render_pipeline: Arc<BaseRenderPipeline>,
    builder: BaseRenderPipelineBuilder,
}

impl MaterialRenderPipeline {
    pub fn new(
        material_render_pipeline_handle: MaterialRenderPipelineHandle,
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> crate::error::Result<MaterialRenderPipeline> {
        let shader_name = ShaderLibrary::get_material_shader_name(material_render_pipeline_handle);

        let mut builder = BaseRenderPipelineBuilder::default();
        builder.targets = vec![Some(ColorTargetState {
            format: texture_format.clone(),
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        builder.shader_name = shader_name;
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Less,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        builder.vertex_buffer_type = Some(VertexBufferType::Interleaved(vec![
            MeshVertex0::type_layout(),
            MeshVertex1::type_layout(),
            MeshVertex2::type_layout(),
        ]));
        builder.primitive = Some(PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        });

        let base_render_pipeline = pool.get(device, shader_library, &builder);

        Ok(MaterialRenderPipeline {
            base_render_pipeline,
            builder,
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
        scissor_rect: Option<glam::UVec4>,
        viewport: Option<Viewport>,
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
            scissor_rect,
            viewport,
        );
    }
    pub fn set_view_mode(
        &mut self,
        view_mode: EViewModeType,
        device: &Device,
        shader_library: &ShaderLibrary,
        pool: &mut BaseRenderPipelinePool,
    ) {
        match view_mode {
            EViewModeType::Wireframe => {
                self.builder.primitive = Some(PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Line,
                    ..Default::default()
                });
            }
            EViewModeType::Lit => {
                self.builder.primitive = Some(PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Fill,
                    ..Default::default()
                });
            }
            EViewModeType::Unlit => todo!(),
        }

        self.base_render_pipeline = pool.get(device, shader_library, &self.builder);
    }
}
