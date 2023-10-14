use super::base_render_pipeline::{BaseRenderPipeline, VertexBufferType};
use crate::brigde_data::gpu_vertex_buffer::GpuVertexBufferImp;
use crate::brigde_data::mesh_vertex::MeshVertex;
use crate::camera::Camera;
use crate::primitive_data::PrimitiveData;
use crate::util;
use glam::{Vec3Swizzles, Vec4Swizzles};
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Constants {
    view: glam::Mat4,
    projection: glam::Mat4,
}

pub struct SkyBoxPipeline {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    vertex_count: u32,
    sampler: Sampler,
    base_render_pipeline: BaseRenderPipeline,
}

impl SkyBoxPipeline {
    pub fn new(device: &Device, texture_format: &wgpu::TextureFormat) -> SkyBoxPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            "sky_box.wgsl",
            texture_format,
            Some(wgpu::DepthStencilState {
                depth_compare: wgpu::CompareFunction::LessEqual,
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            None,
            None,
            None,
            VertexBufferType::Interleaved(MeshVertex::type_layout()),
            None,
        );

        let primitive_data = PrimitiveData::cube();
        let vertex_buffer = crate::util::create_gpu_vertex_buffer_from(
            device,
            &primitive_data.vertices,
            Some("[SkyBoxPipeline] vertex buffer"),
        );
        let index_buffer = crate::util::create_gpu_index_buffer_from(
            device,
            &primitive_data.indices,
            Some("[SkyBoxPipeline] index buffer"),
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        SkyBoxPipeline {
            vertex_buffer,
            index_buffer,
            index_count: primitive_data.indices.len() as u32,
            vertex_count: primitive_data.vertices.len() as u32,
            sampler,
            base_render_pipeline,
        }
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        cube_texture: &wgpu::Texture,
        camera: &Camera,
    ) {
        let view = camera.get_view_matrix();
        let view = glam::mat3(view.x_axis.xyz(), view.y_axis.xyz(), view.z_axis.xyz());
        let mut x_axis = view.x_axis.xyzx();
        x_axis.w = 0.0;
        let mut y_axis = view.y_axis.xyzx();
        y_axis.w = 0.0;
        let mut z_axis = view.z_axis.xyzx();
        z_axis.w = 0.0;

        let view_matrix = glam::mat4(x_axis, y_axis, z_axis, glam::Vec4::W);

        let vshconstants = Constants {
            view: view_matrix,
            projection: camera.get_projection_matrix(),
        };
        let uniform_buf = util::create_gpu_uniform_buffer_from(device, &vshconstants, None);

        let cube_view = cube_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("[SkyBoxPipeline] cube_view"),
            format: Some(wgpu::TextureFormat::Rgba32Float),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        self.base_render_pipeline.draw_resources2(
            device,
            queue,
            vec![
                vec![uniform_buf.as_entire_binding()],
                vec![wgpu::BindingResource::TextureView(&cube_view)],
                vec![wgpu::BindingResource::Sampler(&self.sampler)],
            ],
            &vec![GpuVertexBufferImp {
                vertex_buffers: &vec![&self.vertex_buffer],
                vertex_count: self.vertex_count,
                index_buffer: Some(&self.index_buffer),
                index_count: Some(self.index_count),
            }],
            None,
            None,
            None,
            output_view,
            None,
            Some(depth_view),
        );
    }
}
