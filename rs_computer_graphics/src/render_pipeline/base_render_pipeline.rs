use crate::brigde_data::gpu_vertex_buffer::TGpuVertexBuffer;
use crate::shader::shader_library::ShaderLibrary;
use std::num::NonZeroU32;
use wgpu::*;

#[derive(Debug)]
pub enum VertexBufferType {
    Interleaved(type_layout::TypeLayoutInfo),
    Noninterleaved,
}

pub struct BaseRenderPipeline {
    pub render_pipeline: RenderPipeline,
    bind_group_layouts: Vec<BindGroupLayout>,
    tag: String,
    slots: u32,
}

impl BaseRenderPipeline {
    pub fn new(
        device: &Device,
        file_name: &str,
        texture_format: &TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        multisample: Option<MultisampleState>,
        multiview: Option<NonZeroU32>,
        primitive: Option<PrimitiveState>,
        vertex_buffer_type: VertexBufferType,
    ) -> BaseRenderPipeline {
        let binding = ShaderLibrary::default();
        let shader_library = binding.lock().unwrap();
        let shader = shader_library.get_shader(file_name);
        let reflection = shader_library.get_shader_reflection(file_name);

        let tag: String = file_name.to_owned();

        let bind_group_layout_entrys = reflection.get_bind_group_layout_entrys();

        let mut bind_group_layouts: Vec<BindGroupLayout> = Vec::new();
        for bind_group_layout_entry_vec in bind_group_layout_entrys {
            let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some(&format!("{} bind group layout", tag)),
                entries: bind_group_layout_entry_vec,
            });
            bind_group_layouts.push(bind_group_layout);
        }

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{} pipeline layout", tag)),
            bind_group_layouts: &bind_group_layouts
                .iter()
                .map(|x| x)
                .collect::<Vec<&BindGroupLayout>>(),
            push_constant_ranges: &[],
        });

        let builder = reflection.make_vertex_buffer_layout_builder(vertex_buffer_type);
        let vertex_state_buffers = builder.get_vertex_buffer_layout();

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&format!("{} render pipeline", tag)),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: reflection.get_vs_entry_point(),
                buffers: &vertex_state_buffers,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: reflection.get_fs_entry_point(),
                targets: &[Some(ColorTargetState::from(texture_format.clone()))],
            }),
            primitive: primitive.unwrap_or_default(),
            depth_stencil,
            multisample: multisample.unwrap_or_default(),
            multiview,
        });

        BaseRenderPipeline {
            render_pipeline,
            bind_group_layouts,
            tag,
            slots: vertex_state_buffers.len() as u32,
        }
    }

    pub fn draw_indexed<T>(
        &self,
        device: &Device,
        queue: &Queue,
        entries: Vec<Vec<BindGroupEntry>>,
        mesh_buffers: &[T],
        color_ops: Option<Operations<Color>>,
        depth_ops: Option<Operations<f32>>,
        stencil_ops: Option<Operations<u32>>,
        output_view: &TextureView,
        resolve_target: Option<&TextureView>,
        depth_view: Option<&TextureView>,
    ) where
        T: TGpuVertexBuffer,
    {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some(&format!("{} command encoder", self.tag)),
        });
        {
            let mut depth_stencil_attachment: Option<RenderPassDepthStencilAttachment> = None;
            if let Some(depth_view) = depth_view {
                depth_stencil_attachment = Some(RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(depth_ops.unwrap_or(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    })),
                    stencil_ops: Some(stencil_ops.unwrap_or(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    })),
                });
            }

            let mut bind_groups: Vec<BindGroup> = Vec::new();
            for (entry_vec, bind_group_layout) in entries.iter().zip(self.bind_group_layouts.iter())
            {
                let bind_group = device.create_bind_group(&BindGroupDescriptor {
                    layout: &bind_group_layout,
                    entries: &entry_vec,
                    label: Some(&format!("{} bind group", self.tag)),
                });
                bind_groups.push(bind_group);
            }

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(&format!("{} render pass", self.tag)),
                color_attachments: &[Some(RenderPassColorAttachment {
                    resolve_target,
                    ops: color_ops.unwrap_or(Operations {
                        load: LoadOp::Load,
                        store: true,
                    }),
                    view: output_view,
                })],
                depth_stencil_attachment,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            for (index, bind_group) in bind_groups.iter().enumerate() {
                render_pass.set_bind_group(index as u32, bind_group, &[]);
            }

            for mesh_buffer in mesh_buffers {
                let index_buffer = mesh_buffer.get_index_buffer();
                render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
                for slot in 0..self.slots {
                    let vertex_buffer = mesh_buffer.get_vertex_buffer(slot);
                    render_pass.set_vertex_buffer(slot, vertex_buffer.slice(..));
                }
                render_pass.draw_indexed(0..mesh_buffer.get_index_count(), 0, 0..1);
            }
        }
        queue.submit(Some(encoder.finish()));
    }
}
