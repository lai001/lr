use crate::actor::Actor;
use crate::brigde_data::mesh_vertex::MeshVertex;
use crate::camera::Camera;
use crate::material_type::EMaterialType;
use crate::shader::shader_library::ShaderLibrary;
use crate::static_mesh::StaticMesh;
use crate::virtual_texture::virtual_texture_configuration::VirtualTextureConfiguration;
use crate::{util, VertexBufferLayout};
use wgpu::*;

#[repr(align(16), C)]
#[derive(Clone, Copy, Debug)]
struct Constants {
    model: glam::Mat4,
    view: glam::Mat4,
    projection: glam::Mat4,
    physical_texture_size: u32,
    virtual_texture_size: u32,
    tile_size: u32,
    mipmap_level_bias: f32,
    mipmap_level_scale: f32,
}

pub struct VirtualTextureMeshPipeline {
    render_pipeline: RenderPipeline,
    sampler_bind_group_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
    virtual_texture_configuration: VirtualTextureConfiguration,
    sampler: Sampler,
    sampler_bind_group: BindGroup,
}

impl VirtualTextureMeshPipeline {
    pub fn new(
        device: &Device,
        depth_stencil: Option<DepthStencilState>,
        texture_format: &wgpu::TextureFormat,
        virtual_texture_configuration: VirtualTextureConfiguration,
    ) -> VirtualTextureMeshPipeline {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[VirtualTextureMeshPipeline] texture bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Uint,
                            view_dimension: TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[VirtualTextureMeshPipeline] uniform bind group layout"),
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

        let sampler_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[VirtualTextureMeshPipeline] sampler bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("[VirtualTextureMeshPipeline] pipeline layout"),
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
            .get_shader("virtual_texture_mesh.wgsl");
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
            label: Some("[VirtualTextureMeshPipeline] render pipeline"),
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
            depth_stencil,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let sampler = device.create_sampler(&{
            let mut sampler_descriptor = wgpu::SamplerDescriptor::default();
            // sampler_descriptor.mipmap_filter = wgpu::FilterMode::Linear;
            // sampler_descriptor.min_filter = wgpu::FilterMode::Linear;
            // sampler_descriptor.mag_filter = wgpu::FilterMode::Linear;
            sampler_descriptor
        });
        let sampler_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sampler_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            }],
            label: None,
        });

        VirtualTextureMeshPipeline {
            render_pipeline,
            sampler_bind_group_layout,
            texture_bind_group_layout,
            uniform_bind_group_layout,
            virtual_texture_configuration,
            sampler,
            sampler_bind_group,
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
        depth_ops: Option<wgpu::Operations<f32>>,
        stencil_ops: Option<wgpu::Operations<u32>>,
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
        depth_ops: Option<wgpu::Operations<f32>>,
        stencil_ops: Option<wgpu::Operations<u32>>,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let render_pass_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops,
                stencil_ops,
            };
            let material = {
                match static_mesh.get_material_type() {
                    EMaterialType::Phong(material) => material,
                    EMaterialType::Pbr(_) => panic!(),
                }
            };

            let page_table_texture_view = material.get_page_table_texture_view();
            let physical_texture_view = material.get_physical_texture_view();

            let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&page_table_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&physical_texture_view),
                    },
                ],
                label: None,
            });

            let constants = Constants {
                model: model_matrix.clone(),
                view: camera.get_view_matrix(),
                projection: camera.get_projection_matrix(),
                physical_texture_size: self.virtual_texture_configuration.physical_texture_size,
                virtual_texture_size: self.virtual_texture_configuration.virtual_texture_size,
                tile_size: self.virtual_texture_configuration.tile_size,
                mipmap_level_bias: 0.0,
                mipmap_level_scale: 1.0,
            };
            let uniform_buf = util::create_gpu_uniform_buffer_from(device, &constants, None);
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
                        store: StoreOp::Store,
                    },
                    view: output_view,
                })],
                depth_stencil_attachment: Some(render_pass_depth_stencil_attachment),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &textures_bind_group, &[]);
            render_pass.set_bind_group(2, &self.sampler_bind_group, &[]);

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
