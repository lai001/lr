use std::sync::Arc;

use crate::actor::Actor;
use crate::brigde_data::mesh_vertex::MeshVertex;
use crate::camera::Camera;
use crate::primitive_data::PrimitiveData;
use crate::shader::shader_library::ShaderLibrary;
use crate::{util, VertexBufferLayout};
use glam::{Vec3Swizzles, Vec4Swizzles};
use wgpu::*;

struct Constants {
    view: glam::Mat4,
    projection: glam::Mat4,
}

pub struct SkyBoxPipeline {
    render_pipeline: RenderPipeline,
    sampler_bind_group_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
    depth_ops: Option<Operations<f32>>,
    stencil_ops: Option<Operations<u32>>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl SkyBoxPipeline {
    pub fn new(device: &Device, texture_format: &wgpu::TextureFormat) -> SkyBoxPipeline {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[SkyBoxPipeline] texture bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[SkyBoxPipeline] uniform bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Constants>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        let sampler_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[SkyBoxPipeline] sampler bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("[SkyBoxPipeline] pipeline layout"),
            bind_group_layouts: &[
                &uniform_bind_group_layout,
                &texture_bind_group_layout,
                &sampler_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("sky_box.wgsl");
        let vertex_buffer_layouts = [VertexBufferLayout!(
            MeshVertex,
            [
                VertexFormat::Float32x3,
                VertexFormat::Float32x2,
                VertexFormat::Float32x4,
                VertexFormat::Float32x3,
            ]
        )];
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("[SkyBoxPipeline] render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState::from(texture_format.clone()))],
            }),
            primitive: PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_compare: wgpu::CompareFunction::LessEqual,
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let primitive_data = PrimitiveData::cube();
        let vertex_buffer =
            crate::util::create_gpu_vertex_buffer_from(device, &primitive_data.vertices, None);
        let index_buffer =
            crate::util::create_gpu_index_buffer_from(device, &primitive_data.indices, None);
        SkyBoxPipeline {
            render_pipeline,
            sampler_bind_group_layout,
            texture_bind_group_layout,
            uniform_bind_group_layout,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
            stencil_ops: None,
            vertex_buffer,
            index_buffer,
            index_count: primitive_data.indices.len() as u32,
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
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let render_pass_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: self.depth_ops,
                stencil_ops: self.stencil_ops,
            };

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
            let mut sampler_description = wgpu::SamplerDescriptor::default();
            // sampler_description.compare = Some(CompareFunction::LessEqual);
            let sampler = device.create_sampler(&sampler_description);
            let sampler_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.sampler_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }],
                label: None,
            });

            let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cube_view),
                }],
                label: None,
            });
            // glm::mat4(glm::mat3(camera.get_view_matrix()));
            let view = camera.get_view_matrix();
            let view = glam::mat3(view.x_axis.xyz(), view.y_axis.xyz(), view.z_axis.xyz());
            let mut x_axis = view.x_axis.xyzx();
            x_axis.w = 0.0;
            let mut y_axis = view.y_axis.xyzx();
            y_axis.w = 0.0;
            let mut z_axis = view.z_axis.xyzx();
            z_axis.w = 0.0;
            let mut w_axis = glam::Vec4::W;

            let view_matrix = glam::mat4(x_axis, y_axis, z_axis, w_axis);

            let vshconstants = Constants {
                view: view_matrix,
                projection: camera.get_projection_matrix(),
            };
            let uniform_buf = util::create_gpu_uniform_buffer_from(device, &vshconstants, None);
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
            render_pass.set_bind_group(1, &textures_bind_group, &[]);
            render_pass.set_bind_group(2, &sampler_bind_group, &[]);

            let vertex_buffer = &self.vertex_buffer;
            let index_buffer = &self.index_buffer;
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}
