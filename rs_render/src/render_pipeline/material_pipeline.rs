use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::BaseRenderPipelineBuilder,
    command::MaterialRenderPipelineHandle,
    global_shaders::skeleton_shading::NUM_MAX_BONE,
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::{MeshVertex0, MeshVertex1, MeshVertex2},
    VertexBufferType,
};
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    pub model: glam::Mat4,
    pub id: u32,
    _pad2_0: i32,
    _pad2_1: i32,
    _pad2_2: i32,
    pub bones: [glam::Mat4; NUM_MAX_BONE],
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            model: Default::default(),
            id: Default::default(),
            bones: [glam::Mat4::IDENTITY; NUM_MAX_BONE],
            _pad2_0: Default::default(),
            _pad2_1: Default::default(),
            _pad2_2: Default::default(),
        }
    }
}

pub struct MaterialRenderPipeline {
    base_render_pipeline: BaseRenderPipeline,
}

impl MaterialRenderPipeline {
    pub fn new(
        material_render_pipeline_handle: MaterialRenderPipelineHandle,
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
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

        let base_render_pipeline = BaseRenderPipeline::new(device, shader_library, builder);
        Ok(MaterialRenderPipeline {
            base_render_pipeline,
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
