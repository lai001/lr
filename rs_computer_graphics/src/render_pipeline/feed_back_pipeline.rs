use crate::actor::Actor;
use crate::brigde_data::mesh_vertex::MeshVertex;
use crate::camera::Camera;
use crate::shader::shader_library::ShaderLibrary;
use crate::static_mesh::StaticMesh;
use crate::{util, VertexBufferLayout};
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct VSConstants {
    model: glam::Mat4,
    view: glam::Mat4,
    projection: glam::Mat4,
    physical_texture_size: u32,
    virtual_texture_size: u32,
    tile_size: u32,
    feed_back_texture_width: u32,
    feed_back_texture_height: u32,
}

pub struct FeedBackPipeline {
    render_pipeline: RenderPipeline,
    uniform_bind_group_layout: BindGroupLayout,
}

impl FeedBackPipeline {
    pub fn new(
        device: &Device,
        depth_stencil: Option<DepthStencilState>,
        output_texture_format: &wgpu::TextureFormat,
    ) -> FeedBackPipeline {
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[FeedBackPipeline] uniform bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("[FeedBackPipeline] pipeline layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("feed_back.wgsl");
        let vertex_buffer_layouts = [VertexBufferLayout!(
            MeshVertex,
            [
                VertexFormat::Float32x4,
                VertexFormat::Float32x3,
                VertexFormat::Float32x3,
                VertexFormat::Float32x3,
                VertexFormat::Float32x3,
                VertexFormat::Float32x2,
            ]
        )];
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("[FeedBackPipeline] render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState::from(
                    output_texture_format.clone(),
                ))],
            }),
            primitive: PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        FeedBackPipeline {
            render_pipeline,
            uniform_bind_group_layout,
        }
    }

    pub fn render_actor(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        actor: &Actor,
        camera: &Camera,
        feed_back_texture_width: u32,
        feed_back_texture_height: u32,
        depth_ops: Option<Operations<f32>>,
        stencil_ops: Option<Operations<u32>>,
    ) {
        let model_matrix = actor.get_model_matrix();
        for static_mesh in actor.get_static_meshs() {
            self.render(
                device,
                queue,
                output_view,
                depth_view,
                model_matrix,
                static_mesh,
                camera,
                feed_back_texture_width,
                feed_back_texture_height,
                depth_ops,
                stencil_ops,
            )
        }
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        model_matrix: &glam::Mat4,
        static_mesh: &StaticMesh,
        camera: &Camera,
        feed_back_texture_width: u32,
        feed_back_texture_height: u32,
        depth_ops: Option<Operations<f32>>,
        stencil_ops: Option<Operations<u32>>,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let render_pass_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops,
                stencil_ops,
            };

            let phong_shading_vshconstants = VSConstants {
                model: model_matrix.clone(),
                view: camera.get_view_matrix(),
                projection: camera.get_projection_matrix(),
                physical_texture_size: 4096,
                virtual_texture_size: 512 * 1000,
                tile_size: 256,
                feed_back_texture_width,
                feed_back_texture_height,
            };
            let uniform_buf =
                util::create_gpu_uniform_buffer_from(device, &phong_shading_vshconstants, None);
            let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                }],
                label: None,
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                    view: output_view,
                })],
                depth_stencil_attachment: Some(render_pass_depth_stencil_attachment),
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &uniform_bind_group, &[]);

            let mesh_buffer = static_mesh.get_mesh_buffer();
            let vertex_buffer = mesh_buffer.get_vertex_buffer();
            let index_buffer = mesh_buffer.get_index_buffer();
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..mesh_buffer.get_index_count(), 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}
