use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    global_shaders::shading::ShadingShader,
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::*,
    VertexBufferType,
};
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Constants {
    pub model: glam::Mat4,

    pub diffuse_texture_size: glam::Vec2,
    pub diffuse_texture_max_lod: u32,
    pub is_virtual_diffuse_texture: u32,
    pub specular_texture_size: glam::Vec2,
    pub specular_texture_max_lod: u32,
    pub is_virtual_specular_texture: u32,
    pub id: u32,
    _pad8_0: u32,
    _pad8_1: u32,
    _pad8_2: u32,
}

pub struct ShadingPipeline {
    base_render_pipeline: BaseRenderPipeline,
}

impl ShadingPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        is_noninterleaved: bool,
    ) -> ShadingPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            shader_library,
            &ShadingShader {},
            &[Some(texture_format.clone().into())],
            Some(DepthStencilState {
                depth_compare: CompareFunction::Less,
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            None,
            None,
            Some(PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                ..Default::default()
            }),
            if is_noninterleaved {
                Some(VertexBufferType::Noninterleaved)
            } else {
                Some(VertexBufferType::Interleaved(vec![
                    MeshVertex0::type_layout(),
                    MeshVertex1::type_layout(),
                ]))
            },
            None,
        );

        ShadingPipeline {
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
