use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    global_shaders::virtual_texture_feed_back::SkinMeshVirtualTextureFeedBackShader,
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::{MeshVertex0, MeshVertex2},
    VertexBufferType,
};
use type_layout::TypeLayout;
use wgpu::*;

pub struct SkinMeshVirtualTextureFeedBackPipeline {
    base_render_pipeline: BaseRenderPipeline,
}

impl SkinMeshVirtualTextureFeedBackPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        is_noninterleaved: bool,
    ) -> SkinMeshVirtualTextureFeedBackPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            shader_library,
            &SkinMeshVirtualTextureFeedBackShader {},
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
                cull_mode: None,
                ..Default::default()
            }),
            if is_noninterleaved {
                Some(VertexBufferType::Noninterleaved)
            } else {
                Some(VertexBufferType::Interleaved(vec![
                    MeshVertex0::type_layout(),
                    MeshVertex2::type_layout(),
                ]))
            },
            None,
        );

        SkinMeshVirtualTextureFeedBackPipeline {
            base_render_pipeline,
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        binding_resources: Vec<Vec<BindingResource>>,
        mesh_buffers: &[GpuVertexBufferImp],
    ) {
        self.base_render_pipeline.draw_resources2(
            device,
            queue,
            binding_resources,
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
