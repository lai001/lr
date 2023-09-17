use super::base_render_pipeline::{BaseRenderPipeline, VertexBufferType};
use crate::brigde_data::color_vertex::{ColorVertex, ColorVertexBuffer};
use crate::camera::Camera;
use crate::util;
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Constants {
    view: glam::Mat4,
    projection: glam::Mat4,
}

pub struct Line3DPipeline {
    base_render_pipeline: BaseRenderPipeline,
}

impl Line3DPipeline {
    pub fn new(device: &Device, texture_format: &wgpu::TextureFormat) -> Line3DPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            "line_3d.wgsl",
            texture_format,
            Some(wgpu::DepthStencilState {
                depth_compare: wgpu::CompareFunction::Less,
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            None,
            None,
            Some(PrimitiveState {
                topology: PrimitiveTopology::LineList,
                cull_mode: None,
                polygon_mode: PolygonMode::Line,
                ..Default::default()
            }),
            VertexBufferType::Interleaved(ColorVertex::type_layout()),
            // VertexBufferType::Noninterleaved,
        );
        Line3DPipeline {
            base_render_pipeline,
        }
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        camera: &Camera,
        line3d_buffers: &[ColorVertexBuffer],
    ) {
        let uniform_buf = util::create_gpu_uniform_buffer_from(
            device,
            &Constants {
                view: camera.get_view_matrix(),
                projection: camera.get_projection_matrix(),
            },
            None,
        );

        self.base_render_pipeline.draw_indexed(
            device,
            queue,
            vec![vec![wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }]],
            line3d_buffers,
            None,
            None,
            None,
            output_view,
            None,
            Some(depth_view),
        );
    }
}
