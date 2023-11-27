use super::base_render_pipeline::{BaseRenderPipeline, VertexBufferType};
use crate::brigde_data::gpu_vertex_buffer::GpuVertexBufferImp;
use crate::brigde_data::mesh_vertex::MeshVertex;
use crate::util;
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    model: glam::Mat4,
    view: glam::Mat4,
    projection: glam::Mat4,
}

pub struct NextPhongPipeline {
    base_render_pipeline: BaseRenderPipeline,
    sampler: Sampler,
}

impl NextPhongPipeline {
    pub fn new(
        device: &Device,
        texture_format: &TextureFormat,
        is_noninterleaved: bool,
    ) -> NextPhongPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
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

        NextPhongPipeline {
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
        let uniform_buf = util::create_gpu_uniform_buffer_from(device, constants, None);

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
