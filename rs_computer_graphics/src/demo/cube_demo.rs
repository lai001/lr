use std::{borrow::Borrow, f32::consts::E, sync::Arc};

use glam::{Vec3Swizzles, Vec4Swizzles};

use crate::{
    brigde_data::mesh_vertex::MeshVertex, camera::Camera, resource_manager::ResourceManager,
    shader::shader_library::ShaderLibrary,
};

pub struct CubeDemo {
    pub model_matrix: glam::Mat4,
    pub vertex_data: Vec<MeshVertex>,
    render_pipeline: wgpu::RenderPipeline,
    render_pipeline_write: Option<wgpu::RenderPipeline>,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    depth_texture: wgpu::Texture,
    bind_group_layout: wgpu::BindGroupLayout,
    color_texture: wgpu::Texture,
}

impl CubeDemo {
    pub fn create_color_grid_texture_resource(size: usize) -> Vec<u8> {
        let cache_image = ResourceManager::default()
            .lock()
            .unwrap()
            .get_cache_image("ColorGrid.png");
        let is_flipv = false;
        match cache_image {
            Some(cache_image) => {
                let dynamic_image: image::DynamicImage;
                if is_flipv {
                    dynamic_image = cache_image.flipv();
                } else {
                    dynamic_image = image::DynamicImage::ImageRgba8(cache_image.to_rgba8());
                }
                let dynamic_image = dynamic_image.resize(
                    size as u32,
                    size as u32,
                    image::imageops::FilterType::Nearest,
                );
                let image_buffers = dynamic_image.into_rgba8();

                let mut texture_buffer: Vec<u8> = vec![0; (4 * size * size).try_into().unwrap()];
                texture_buffer.copy_from_slice(&image_buffers);
                texture_buffer
            }
            None => panic!(),
        }
    }

    pub fn vertex(position: glam::Vec4, tex_coord: glam::Vec2) -> MeshVertex {
        MeshVertex {
            position: position.xyz(),
            tex_coord,
            vertex_color: glam::vec4(0.0, 0.0, 0.0, 0.0),
            normal: glam::vec3(0.0, 0.0, 1.0),
        }
    }

    fn append_component(vector: &glam::Vec3) -> glam::Vec4 {
        let mut ret = vector.xyzx();
        ret.w = 1.0;
        ret
    }

    pub fn create_vertices() -> (Vec<MeshVertex>, Vec<u16>) {
        let base_plane_data = [
            Self::vertex(glam::vec4(-1.0, 1.0, 0.0, 1.0), glam::vec2(0.0, 0.0)),
            Self::vertex(glam::vec4(1.0, 1.0, 0.0, 1.0), glam::vec2(1.0, 0.0)),
            Self::vertex(glam::vec4(1.0, -1.0, 0.0, 1.0), glam::vec2(1.0, 1.0)),
            Self::vertex(glam::vec4(-1.0, -1.0, 0.0, 1.0), glam::vec2(0.0, 1.0)),
        ];

        let front_plane_data = base_plane_data.map(|item| {
            let translation = glam::Mat4::from_translation(glam::Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            });
            Self::vertex(
                translation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let back_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_y(180.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let left_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_y(-90.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let right_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_y(90.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let top_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_x(-90.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let bottom_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_x(90.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let front_plane_index: Vec<u16> = [2, 1, 0, 3, 2, 0].to_vec();
        let back_plane_index: Vec<u16> = front_plane_index.iter().map(|item| item + 4).collect();
        let left_plane_index: Vec<u16> = back_plane_index.iter().map(|item| item + 4).collect();
        let right_plane_index: Vec<u16> = left_plane_index.iter().map(|item| item + 4).collect();
        let top_plane_index: Vec<u16> = right_plane_index.iter().map(|item| item + 4).collect();
        let bottom_plane_index: Vec<u16> = top_plane_index.iter().map(|item| item + 4).collect();

        (
            [
                front_plane_data,
                back_plane_data,
                left_plane_data,
                right_plane_data,
                top_plane_data,
                bottom_plane_data,
            ]
            .concat()
            .to_vec(),
            [
                front_plane_index,
                back_plane_index,
                left_plane_index,
                right_plane_index,
                top_plane_index,
                bottom_plane_index,
            ]
            .concat()
            .to_vec(),
        )
    }

    pub fn new(
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
        queue: &wgpu::Queue,
        window_width: u32,
        window_height: u32,
    ) -> CubeDemo {
        let size = 1024_u32;
        let color_texture_resource = Self::create_color_grid_texture_resource(size as usize);

        let (vertex_data, index_data) = Self::create_vertices();
        let model_matrix = glam::Mat4::from_translation(glam::Vec3 {
            x: 0.0,
            y: 0.0,
            z: -5.0,
        });

        let unsafe_vertex_data_raw_buffer: &[u8] = unsafe {
            std::slice::from_raw_parts(
                vertex_data.as_ptr() as *const u8,
                vertex_data.len() * std::mem::size_of::<MeshVertex>(),
            )
        };

        let unsafe_index_data_raw_buffer: &[u8] = unsafe {
            std::slice::from_raw_parts(
                index_data.as_ptr() as *const u8,
                index_data.len() * std::mem::size_of::<u16>(),
            )
        };

        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };
        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            color_texture.as_image_copy(),
            &color_texture_resource,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size * 4),
                rows_per_image: None,
            },
            texture_extent,
        );

        let depth_texture_extent = wgpu::Extent3d {
            width: window_width,
            height: window_height,
            depth_or_array_layers: 1,
        };
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: depth_texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("cube.wgsl");

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let vertex_size = std::mem::size_of::<MeshVertex>();
        let vertex_buffer_layouts = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: (std::mem::size_of::<f32>() * 3) as u64,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: (std::mem::size_of::<f32>() * 5) as u64,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: (std::mem::size_of::<f32>() * 9) as u64,
                    shader_location: 3,
                },
            ],
        }];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(texture_format.clone().into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_compare: wgpu::CompareFunction::Less,
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let render_pipeline_write = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            let pipeline_wire = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &vertex_buffer_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_wire",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: texture_format.clone(),
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                operation: wgpu::BlendOperation::Add,
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            },
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Line,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
            Some(pipeline_wire)
        } else {
            None
        };
        let vertex_buf = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: unsafe_vertex_data_raw_buffer,
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        let index_buf = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: unsafe_index_data_raw_buffer,
                usage: wgpu::BufferUsages::INDEX,
            },
        );
        CubeDemo {
            render_pipeline,
            render_pipeline_write,
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            vertex_data,
            depth_texture,
            model_matrix,
            bind_group_layout,
            color_texture,
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        output_view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        camera: &Camera,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let depth_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let render_pass_depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            };

            let texture_view = self
                .color_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

            let mvp = camera.get_projection_matrix() * camera.get_view_matrix() * self.model_matrix;
            let matrix_ref: &[f32; 16] = mvp.as_ref();
            let unsafe_uniform_raw_buffer: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    matrix_ref.as_ptr() as *const u8,
                    matrix_ref.len() * std::mem::size_of::<f32>(),
                )
            };
            let uniform_buf = wgpu::util::DeviceExt::create_buffer_init(
                device,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform Buffer"),
                    contents: unsafe_uniform_raw_buffer,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                },
            );
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: None,
            });

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                    view: output_view,
                })],
                depth_stencil_attachment: Some(render_pass_depth_stencil_attachment),
            });
            // rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            // rpass.pop_debug_group();
            // rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
            if let Some(ref pipe) = self.render_pipeline_write {
                rpass.set_pipeline(pipe);
                rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}
