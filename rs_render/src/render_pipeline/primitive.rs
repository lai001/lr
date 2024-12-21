use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    global_shaders::{global_shader::GlobalShader, primitive::PrimitiveShader},
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::MeshVertex3,
    VertexBufferType,
};
use std::{collections::HashMap, sync::Arc};
use type_layout::TypeLayout;
use wgpu::*;

pub struct PrimitiveRenderPipeline {
    base_render_pipelines: HashMap<PrimitiveState, Arc<BaseRenderPipeline>>,
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

        // builder.primitive = Some(PrimitiveState {
        //     topology: PrimitiveTopology::LineList,
        //     cull_mode: None,
        //     polygon_mode: PolygonMode::Line,
        //     ..Default::default()
        // });

        let mut base_render_pipelines = HashMap::new();
        for topology in vec![
            PrimitiveTopology::PointList,
            PrimitiveTopology::LineList,
            PrimitiveTopology::LineStrip,
            PrimitiveTopology::TriangleList,
            PrimitiveTopology::TriangleStrip,
        ] {
            let mut polygon_modes = vec![PolygonMode::Fill];
            if device.features().contains(Features::POLYGON_MODE_LINE) {
                polygon_modes.push(PolygonMode::Line);
            }
            if device.features().contains(Features::POLYGON_MODE_POINT) {
                polygon_modes.push(PolygonMode::Point);
            }
            for polygon_mode in polygon_modes {
                let primitive = PrimitiveState {
                    topology,
                    cull_mode: None,
                    polygon_mode,
                    ..Default::default()
                };
                builder.primitive = Some(primitive.clone());
                let base_render_pipeline = pool.get(device, shader_library, &builder);
                base_render_pipelines.insert(primitive, base_render_pipeline);
            }
        }

        Ok(PrimitiveRenderPipeline {
            base_render_pipelines,
            _builder: builder,
        })
    }

    fn default_primitive_state() -> PrimitiveState {
        PrimitiveState {
            topology: PrimitiveTopology::LineList,
            cull_mode: None,
            polygon_mode: PolygonMode::Line,
            ..Default::default()
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
        self.base_render_pipelines
            .get(&Self::default_primitive_state())
            .expect("Not null")
            .draw_resources(
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

    pub fn get_base_render_pipeline(
        &self,
        primitive_state: Option<&PrimitiveState>,
    ) -> Option<&BaseRenderPipeline> {
        let pipeline = self
            .base_render_pipelines
            .get(primitive_state.unwrap_or(&Self::default_primitive_state()));
        pipeline.as_ref().map(|v| &***v)
    }
}
