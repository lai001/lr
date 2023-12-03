use crate::{
    brigde_data::image2d_vertex::Image2DVertex, shader::shader_library::ShaderLibrary, util,
    VertexBufferLayout,
};
use rs_foundation::cast_to_raw_buffer;
use wgpu::*;

pub struct YUV420pPipeline {
    render_pipeline: RenderPipeline,
    texture_bind_group_layout: BindGroupLayout,
    sampler_bind_group_layout: BindGroupLayout,
    index_buf: wgpu::Buffer,
}

impl YUV420pPipeline {
    pub fn new(device: &Device, texture_format: &wgpu::TextureFormat) -> YUV420pPipeline {
        let shader = ShaderLibrary::default()
            .lock()
            .unwrap()
            .get_shader("yuv420p.wgsl");

        // let vertex_buffer_layouts = [VertexBufferLayout {
        //     array_stride: std::mem::size_of::<Image2DVertex>() as BufferAddress,
        //     step_mode: VertexStepMode::Vertex,
        //     attributes: &[
        //         VertexAttribute {
        //             format: VertexFormat::Float32x2,
        //             offset: 0,
        //             shader_location: 0,
        //         },
        //         VertexAttribute {
        //             format: VertexFormat::Float32x2,
        //             offset: 4 * 2,
        //             shader_location: 1,
        //         },
        //     ],
        // }];

        let vertex_buffer_layouts = [VertexBufferLayout!(
            Image2DVertex,
            [VertexFormat::Float32x2, VertexFormat::Float32x2]
        )];

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[YUV420pPipeline] texture bind group layout"),
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
                ],
            });

        let sampler_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[YUV420pPipeline] sampler bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                }],
            });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("[YUV420pPipeline] pipeline layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &sampler_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("[YUV420pPipeline] render pipeline"),
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
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        YUV420pPipeline {
            render_pipeline,
            texture_bind_group_layout,
            index_buf: util::create_gpu_index_buffer_from(
                device,
                &(vec![0 as u32, 1, 2, 0, 2, 3]),
                Some("[YUV420pPipeline] index buffer"),
            ),
            sampler_bind_group_layout,
        }
    }

    pub fn render(
        &self,
        vertex: Vec<Image2DVertex>,
        device: &Device,
        output_view: &TextureView,
        queue: &Queue,
        y_texture: &Texture,
        cb_texture: &Texture,
        cr_texture: &Texture,
    ) {
        assert_eq!(vertex.len(), 4);
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("[YUV420pPipeline] command encoder"),
        });
        {
            let vertex_buf = wgpu::util::DeviceExt::create_buffer_init(
                device,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("[YUV420pPipeline] vertex buffer"),
                    contents: cast_to_raw_buffer(&vertex),
                    usage: BufferUsages::VERTEX,
                },
            );
            let y_texture_view = y_texture.create_view(&TextureViewDescriptor::default());
            let cb_texture_view = cb_texture.create_view(&TextureViewDescriptor::default());
            let cr_texture_view = cr_texture.create_view(&TextureViewDescriptor::default());
            let sampler = device.create_sampler(&SamplerDescriptor::default());

            let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&y_texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&cb_texture_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&cr_texture_view),
                    },
                ],
                label: Some("[YUV420pPipeline] texture bind group"),
            });

            let sampler_bind_group = device.create_bind_group(&BindGroupDescriptor {
                layout: &self.sampler_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&sampler),
                }],
                label: Some("[YUV420pPipeline] sampler bind group"),
            });

            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("[YUV420pPipeline] render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                    view: output_view,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &texture_bind_group, &[]);
            rpass.set_bind_group(1, &sampler_bind_group, &[]);
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            rpass.set_vertex_buffer(0, vertex_buf.slice(..));
            rpass.draw_indexed(0..6, 0, 0..1);
            rpass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}
