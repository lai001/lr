use crate::{
    actor::Actor,
    brigde_data::mesh_vertex::MeshVertex,
    // brigde_data::mesh_vertex::PBRMeshVertex,
    camera::Camera,
    light::{DirectionalLight, PointLight, SpotLight},
    material_type::EMaterialType,
    shader::shader_library::ShaderLibrary,
    static_mesh::StaticMesh,
};
use crate::{util, VertexBufferLayout};
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Constants {
    directional_light: DirectionalLight,
    point_light: PointLight,
    spot_light: SpotLight,
    model: glam::Mat4,
    view: glam::Mat4,
    projection: glam::Mat4,
    view_position: glam::Vec3,
    roughness_factor: f32,
    metalness_factor: f32,
    _padding3: [u32; 3],
}

pub struct PBRPipeline {
    render_pipeline: RenderPipeline,
    sampler_bind_group_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
    depth_ops: Option<Operations<f32>>,
    stencil_ops: Option<Operations<u32>>,
    depth_stencil: Option<DepthStencilState>,
}

impl PBRPipeline {
    pub fn new(
        device: &Device,
        depth_stencil: Option<DepthStencilState>,
        texture_format: &wgpu::TextureFormat,
    ) -> PBRPipeline {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[PBRPipeline] texture bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 6,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[PBRPipeline] uniform bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Constants>() as u64
                        ),
                        // min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let sampler_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[PBRPipeline] sampler bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("[PBRPipeline] pipeline layout"),
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
            .get_shader("pbr.wgsl");
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
            label: Some("[PBRPipeline] render pipeline"),
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
            depth_stencil: {
                match depth_stencil {
                    Some(ref x) => Some(x.clone()),
                    None => None,
                }
            },
            multisample: MultisampleState::default(),
            multiview: None,
        });
        PBRPipeline {
            render_pipeline,
            sampler_bind_group_layout,
            texture_bind_group_layout,
            uniform_bind_group_layout,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
            stencil_ops: None,
            depth_stencil,
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
        roughness_factor: f32,
        metalness_factor: f32,
        directional_light: DirectionalLight,
        point_light: PointLight,
        spot_light: SpotLight,
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
                roughness_factor,
                metalness_factor,
                directional_light,
                point_light,
                spot_light,
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
        roughness_factor: f32,
        metalness_factor: f32,
        directional_light: DirectionalLight,
        point_light: PointLight,
        spot_light: SpotLight,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let render_pass_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: self.depth_ops,
                stencil_ops: self.stencil_ops,
            };
            let material = {
                match static_mesh.get_material_type() {
                    EMaterialType::Phong(_) => panic!(),
                    EMaterialType::Pbr(material) => material,
                }
            };
            let albedo_texture_view = material.get_albedo_texture_view();
            let metallic_texture_view = material.get_metallic_texture_view();
            let brdflut_texture_view = material.get_brdflut_texture_view();
            let irradiance_texture_view = material.get_irradiance_texture_view();
            let roughness_texture_view = material.get_roughness_texture_view();
            let pre_filter_cube_map_texture_view = material.get_pre_filter_cube_map_texture_view();
            let normal_texture_view = material.get_normal_texture_view();

            let sampler_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.sampler_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(
                            &device.create_sampler(&wgpu::SamplerDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &device.create_sampler(&wgpu::SamplerDescriptor::default()),
                        ),
                    },
                ],
                label: None,
            });

            let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&albedo_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&normal_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&metallic_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&roughness_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&brdflut_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(
                            &pre_filter_cube_map_texture_view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&irradiance_texture_view),
                    },
                ],
                label: None,
            });

            let constants = Constants {
                directional_light,
                point_light,
                spot_light,
                model: *model_matrix,
                view: camera.get_view_matrix(),
                projection: camera.get_projection_matrix(),
                view_position: camera.get_world_location(),
                roughness_factor,
                metalness_factor,
                _padding3: [0, 0, 0],
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
                        // load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
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
