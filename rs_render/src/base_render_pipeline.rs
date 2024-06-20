use crate::base_render_pipeline_pool::BaseRenderPipelineBuilder;
use crate::bind_group_layout_entry_hook::EBindGroupLayoutEntryHookType;
use crate::command::Viewport;
use crate::gpu_vertex_buffer::{EDrawCallType, GpuVertexBufferImp};
use crate::reflection::{EPipelineType, VertexBufferLayoutBuilder};
use crate::shader_library::ShaderLibrary;
use wgpu::*;

pub struct ColorAttachment<'a> {
    pub color_ops: Option<Operations<Color>>,
    pub view: &'a TextureView,
    pub resolve_target: Option<&'a TextureView>,
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
        shader_library: &ShaderLibrary,
        base_render_pipeline_builder: BaseRenderPipelineBuilder,
    ) -> BaseRenderPipeline {
        let BaseRenderPipelineBuilder {
            shader_name,
            targets,
            depth_stencil,
            multisample,
            multiview,
            primitive,
            vertex_buffer_type,
            hooks,
            vertex_layout_hooks,
        } = base_render_pipeline_builder;

        let tag = shader_name.as_ref();
        let shader = shader_library.get_shader(tag);
        let reflection = shader_library.get_shader_reflection(tag);
        let EPipelineType::Render(vs, fs) = reflection.get_pipeline_type() else {
            panic!()
        };

        let mut bind_group_layouts: Vec<BindGroupLayout> = Vec::new();

        match hooks {
            Some(hooks) => {
                let mut bind_group_layout_entrys =
                    reflection.get_bind_group_layout_entrys().to_vec();

                for (x, bind_group_layout_entry_vec) in
                    bind_group_layout_entrys.iter_mut().enumerate()
                {
                    for (y, entry) in bind_group_layout_entry_vec.iter_mut().enumerate() {
                        if let Some(hook_value) = hooks.get(&glam::uvec2(x as u32, y as u32)) {
                            match (hook_value, &mut entry.ty) {
                                (
                                    EBindGroupLayoutEntryHookType::Uniform(uniform),
                                    BindingType::Buffer {
                                        has_dynamic_offset,
                                        min_binding_size,
                                        ..
                                    },
                                ) => {
                                    entry.count = uniform.count;
                                    *has_dynamic_offset = uniform.has_dynamic_offset;
                                    *min_binding_size = uniform.min_binding_size;
                                }
                                (
                                    EBindGroupLayoutEntryHookType::TextureSampleType(
                                        texture_sample_type,
                                    ),
                                    BindingType::Texture { sample_type, .. },
                                ) => {
                                    entry.count = texture_sample_type.count;
                                    *sample_type = texture_sample_type.sample_type;
                                }
                                (
                                    EBindGroupLayoutEntryHookType::SamplerBindingType(
                                        sampler_binding_type,
                                    ),
                                    BindingType::Sampler(sampler),
                                ) => {
                                    *sampler = *sampler_binding_type;
                                }
                                _ => todo!(),
                            }
                        }
                    }
                    log::trace!(
                        "bind_group_layout_entry_vec: {:?}",
                        bind_group_layout_entry_vec
                    );
                    let bind_group_layout =
                        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                            label: Some(&format!("{} bind group layout", tag)),
                            entries: bind_group_layout_entry_vec,
                        });
                    bind_group_layouts.push(bind_group_layout);
                }
            }
            None => {
                let bind_group_layout_entrys = reflection.get_bind_group_layout_entrys();
                for bind_group_layout_entry_vec in bind_group_layout_entrys {
                    let bind_group_layout =
                        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                            label: Some(&format!("{} bind group layout", tag)),
                            entries: bind_group_layout_entry_vec,
                        });
                    bind_group_layouts.push(bind_group_layout);
                }
            }
        }

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{} pipeline layout", tag)),
            bind_group_layouts: &bind_group_layouts
                .iter()
                .map(|x| x)
                .collect::<Vec<&BindGroupLayout>>(),
            push_constant_ranges: &[],
        });
        let mut vertex_buffer_layouts: Vec<VertexBufferLayout> = vec![];
        let mut builder: Option<VertexBufferLayoutBuilder> = None;
        if let Some(vertex_buffer_type) = vertex_buffer_type {
            builder = Some(reflection.make_vertex_buffer_layout_builder(vertex_buffer_type));
        }
        if let Some(builder) = &builder {
            vertex_buffer_layouts = builder.get_vertex_buffer_layout();
        }
        if let Some(vertex_layout_hooks) = vertex_layout_hooks {
            for (location, step_mode) in vertex_layout_hooks {
                if let Some(vertex_buffer_layout) = vertex_buffer_layouts.get_mut(location) {
                    vertex_buffer_layout.step_mode = step_mode;
                }
            }
        }

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&format!("{} render pipeline", tag)),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: &vs.name,
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: &fs.name,
                targets: &targets,
            }),
            primitive: primitive.unwrap_or_default(),
            depth_stencil,
            multisample: multisample.unwrap_or_default(),
            multiview,
        });

        BaseRenderPipeline {
            render_pipeline,
            bind_group_layouts,
            tag: tag.to_string(),
            slots: vertex_buffer_layouts.len() as u32,
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        entries: Vec<Vec<BindGroupEntry>>,
        mesh_buffers: &[GpuVertexBufferImp],
        color_attachments: &[ColorAttachment],
        depth_ops: Option<Operations<f32>>,
        stencil_ops: Option<Operations<u32>>,
        depth_view: Option<&TextureView>,
        scissor_rect: Option<glam::UVec4>,
        viewport: Option<Viewport>,
    ) -> SubmissionIndex {
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
                        store: StoreOp::Store,
                    })),
                    stencil_ops: Some(stencil_ops.unwrap_or(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: StoreOp::Store,
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

            let mut render_pass_color_attachments: Vec<Option<RenderPassColorAttachment>> =
                Vec::new();
            for x in color_attachments {
                render_pass_color_attachments.push(Some(RenderPassColorAttachment {
                    ops: x.color_ops.unwrap_or(Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    }),
                    view: x.view,
                    resolve_target: x.resolve_target,
                }));
            }
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(&format!("{} render pass", self.tag)),
                color_attachments: &render_pass_color_attachments,
                depth_stencil_attachment,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let Some(rect) = scissor_rect {
                render_pass.set_scissor_rect(rect.x, rect.y, rect.z, rect.w);
            }
            if let Some(viewport) = viewport {
                let rect = &viewport.rect;
                let depth_range = &viewport.depth_range;
                render_pass.set_viewport(
                    rect.x,
                    rect.y,
                    rect.z,
                    rect.w,
                    depth_range.start,
                    depth_range.end,
                );
            }
            render_pass.set_pipeline(&self.render_pipeline);
            for (index, bind_group) in bind_groups.iter().enumerate() {
                render_pass.set_bind_group(index as u32, bind_group, &[]);
            }

            for mesh_buffer in mesh_buffers {
                if self.slots as usize != mesh_buffer.vertex_buffers.len() {
                    panic!(
                        "{}, required slots {} != vertex buffers len {}",
                        self.tag,
                        self.slots,
                        mesh_buffer.vertex_buffers.len()
                    );
                }
                for (slot, vertex_buffer) in mesh_buffer.vertex_buffers.iter().enumerate() {
                    render_pass.set_vertex_buffer(slot as u32, vertex_buffer.slice(..));
                }
                if let (Some(index_buffer), Some(index_count)) =
                    (mesh_buffer.index_buffer, mesh_buffer.index_count)
                {
                    render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
                    match &mesh_buffer.draw_type {
                        EDrawCallType::MultiDrawIndirect(multi_draw_indirect) => {
                            render_pass.multi_draw_indexed_indirect(
                                multi_draw_indirect.indirect_buffer,
                                multi_draw_indirect.indirect_offset,
                                multi_draw_indirect.count,
                            );
                        }
                        EDrawCallType::Draw(draw) => {
                            render_pass.draw_indexed(0..index_count, 0, draw.instances.clone());
                        }
                    }
                } else {
                    match &mesh_buffer.draw_type {
                        EDrawCallType::MultiDrawIndirect(multi_draw_indirect) => {
                            render_pass.multi_draw_indirect(
                                multi_draw_indirect.indirect_buffer,
                                multi_draw_indirect.indirect_offset,
                                multi_draw_indirect.count,
                            );
                        }
                        EDrawCallType::Draw(draw) => {
                            render_pass.draw(0..mesh_buffer.vertex_count, draw.instances.clone());
                        }
                    }
                }
            }
        }
        queue.submit(Some(encoder.finish()))
    }

    pub fn draw_resources(
        &self,
        device: &Device,
        queue: &Queue,
        binding_resources: Vec<Vec<BindingResource>>,
        mesh_buffers: &[GpuVertexBufferImp],
        color_attachments: &[ColorAttachment],
        depth_ops: Option<Operations<f32>>,
        stencil_ops: Option<Operations<u32>>,
        depth_view: Option<&TextureView>,
        scissor_rect: Option<glam::UVec4>,
        viewport: Option<Viewport>,
    ) -> SubmissionIndex {
        let entries = binding_resources
            .iter()
            .map(|x| {
                x.iter()
                    .enumerate()
                    .map(|(binding, resource)| wgpu::BindGroupEntry {
                        binding: binding as u32,
                        resource: resource.clone(),
                    })
                    .collect()
            })
            .collect();
        self.draw(
            device,
            queue,
            entries,
            mesh_buffers,
            color_attachments,
            depth_ops,
            stencil_ops,
            depth_view,
            scissor_rect,
            viewport,
        )
    }

    pub fn get_tag(&self) -> &str {
        &self.tag
    }
}
