use crate::{
    base_render_pipeline::BaseRenderPipeline,
    global_shaders::virtual_texture_feed_back::VirtualTextureFeedBackShader, gpu_buffer,
    gpu_vertex_buffer::GpuVertexBufferImp, shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::MeshVertex, VertexBufferType,
};
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    pub model: glam::Mat4,
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
    pub physical_texture_size: u32,
    pub virtual_texture_size: u32,
    pub tile_size: u32,
    pub feed_back_texture_width: u32,
    pub feed_back_texture_height: u32,
    pub mipmap_level_bias: f32,
    pub mipmap_level_scale: f32,
    pub feedback_bias: f32,
}

pub struct VirtualTextureFeedBackPipeline {
    base_render_pipeline: BaseRenderPipeline,
}

impl VirtualTextureFeedBackPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        is_noninterleaved: bool,
    ) -> VirtualTextureFeedBackPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            shader_library,
            &VirtualTextureFeedBackShader {},
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
                Some(VertexBufferType::Interleaved(MeshVertex::type_layout()))
            },
            None,
        );

        VirtualTextureFeedBackPipeline {
            base_render_pipeline,
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
        let uniform_buf = gpu_buffer::uniform::from(device, constants, None);

        self.base_render_pipeline.draw_resources2(
            device,
            queue,
            vec![vec![uniform_buf.as_entire_binding()]],
            mesh_buffers,
            None,
            None,
            None,
            output_view,
            None,
            Some(depth_view),
        );
    }
}
