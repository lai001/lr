use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    global_shaders::shading::ShadingShader,
    gpu_buffer,
    gpu_vertex_buffer::GpuVertexBufferImp,
    sampler_cache::SamplerCache,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::*,
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
    pub physical_texture_size: glam::Vec2,
    pub diffuse_texture_size: glam::Vec2,
    pub diffuse_texture_max_lod: u32,
    pub is_virtual_diffuse_texture: u32,
    pub specular_texture_size: glam::Vec2,
    pub specular_texture_max_lod: u32,
    pub is_virtual_specular_texture: u32,
    pub tile_size: f32,
    pub is_enable_virtual_texture: i32,
}

pub struct ShadingPipeline {
    base_render_pipeline: BaseRenderPipeline,
    sampler: Arc<Sampler>,
}

impl ShadingPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        is_noninterleaved: bool,
        sampler_cache: &mut SamplerCache,
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

        let sampler = sampler_cache.create_sampler(device, &SamplerDescriptor::default());

        ShadingPipeline {
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
        physical_texture_view: &TextureView,
        page_table_texture_view: &TextureView,
    ) {
        let uniform_buf =
            gpu_buffer::uniform::from(device, constants, Some("ShadingPipeline.constants"));

        self.base_render_pipeline.draw_resources2(
            device,
            queue,
            vec![
                vec![uniform_buf.as_entire_binding()],
                vec![
                    BindingResource::TextureView(&diffuse_texture_view),
                    BindingResource::TextureView(&specular_texture_view),
                ],
                vec![
                    BindingResource::TextureView(&physical_texture_view),
                    BindingResource::TextureView(&page_table_texture_view),
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
