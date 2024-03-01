use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    global_shaders::phong::PhongShader,
    gpu_buffer,
    gpu_vertex_buffer::GpuVertexBufferImp,
    sampler_cache::SamplerCache,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::MeshVertex,
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

pub struct PhongPipeline {
    base_render_pipeline: BaseRenderPipeline,
    sampler: Arc<Sampler>,
}

impl PhongPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        is_noninterleaved: bool,
        sampler_cache: &mut SamplerCache,
    ) -> PhongPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            shader_library,
            &PhongShader {},
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
                Some(VertexBufferType::Interleaved(MeshVertex::type_layout()))
            },
            None,
        );

        let sampler = sampler_cache.create_sampler(device, &SamplerDescriptor::default());

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
