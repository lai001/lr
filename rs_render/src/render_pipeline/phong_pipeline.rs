use crate::{
    base_render_pipeline::BaseRenderPipeline, gpu_buffer, gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary, vertex_data_type::mesh_vertex::MeshVertex, VertexBufferType,
};
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    pub model: glam::Mat4,
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
}

pub struct PhongPipeline {
    base_render_pipeline: BaseRenderPipeline,
    sampler: Sampler,
}

impl PhongPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        is_noninterleaved: bool,
    ) -> PhongPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            shader_library,
            "phong.wgsl",
            texture_format,
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
                VertexBufferType::Noninterleaved
            } else {
                VertexBufferType::Interleaved(MeshVertex::type_layout())
            },
            None,
        );
        let sampler = device.create_sampler(&SamplerDescriptor::default());

        PhongPipeline {
            base_render_pipeline,
            sampler,
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
        diffuse_texture_view: &TextureView,
        specular_texture_view: &TextureView,
    ) {
        let uniform_buf = gpu_buffer::uniform::from(device, constants, None);

        self.base_render_pipeline.draw_resources2(
            device,
            queue,
            vec![
                vec![uniform_buf.as_entire_binding()],
                vec![
                    BindingResource::TextureView(&diffuse_texture_view),
                    BindingResource::TextureView(&specular_texture_view),
                ],
                vec![BindingResource::Sampler(&self.sampler)],
            ],
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
