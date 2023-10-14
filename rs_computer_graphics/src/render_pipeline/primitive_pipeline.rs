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

pub struct PrimitivePipeline {
    base_render_pipeline: BaseRenderPipeline,
}

impl PrimitivePipeline {
    pub fn new(
        device: &Device,
        texture_format: &wgpu::TextureFormat,
        topology: PrimitiveTopology,
        polygon_mode: PolygonMode,
        is_noninterleaved: bool,
    ) -> PrimitivePipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            "primitive.wgsl",
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
                topology,
                cull_mode: None,
                polygon_mode,
                ..Default::default()
            }),
            if is_noninterleaved {
                VertexBufferType::Noninterleaved
            } else {
                VertexBufferType::Interleaved(ColorVertex::type_layout())
            },
            None,
        );
        PrimitivePipeline {
            base_render_pipeline,
        }
    }

    pub fn draw(
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

        self.base_render_pipeline.draw_resources(
            device,
            queue,
            vec![vec![uniform_buf.as_entire_binding()]],
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
