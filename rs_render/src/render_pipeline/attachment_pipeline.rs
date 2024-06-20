use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    global_shaders::{attachment::AttachmentShader, global_shader::GlobalShader},
    gpu_vertex_buffer::{Draw, EDrawCallType, GpuVertexBufferImp},
    shader_library::ShaderLibrary,
};
use std::sync::Arc;
use wgpu::*;

pub struct ClearColor<'a> {
    pub view: &'a TextureView,
    pub resolve_target: Option<&'a TextureView>,
    pub color: wgpu::Color,
}

pub struct ClearDepth<'a> {
    pub view: &'a TextureView,
}

pub struct ClearAll<'a> {
    pub clear_color: ClearColor<'a>,
    pub clear_depth: ClearDepth<'a>,
}

pub enum EClearType<'a> {
    Depth(ClearDepth<'a>),
    Color(ClearColor<'a>),
    Both(ClearAll<'a>),
}

pub struct AttachmentPipeline {
    base_render_pipeline: Arc<BaseRenderPipeline>,
    multisample_pipeline: Arc<BaseRenderPipeline>,
    clear_depth_pipeline: Arc<BaseRenderPipeline>,
}

impl AttachmentPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> AttachmentPipeline {
        let mut builder = BaseRenderPipelineBuilder::default();
        builder.shader_name = AttachmentShader {}.get_name();
        builder.targets = vec![Some(texture_format.clone().into())];
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Never,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        builder.primitive = Some(PrimitiveState {
            cull_mode: None,
            ..Default::default()
        });
        let base_render_pipeline = pool.get(device, shader_library, &builder);
        builder.multisample = Some(MultisampleState {
            count: 4,
            ..Default::default()
        });
        let multisample_pipeline = pool.get(device, shader_library, &builder);
        builder.multisample = None;
        builder.targets = vec![];
        let clear_depth_pipeline = pool.get(device, shader_library, &builder);
        AttachmentPipeline {
            base_render_pipeline,
            clear_depth_pipeline,
            multisample_pipeline,
        }
    }

    pub fn draw(&self, device: &Device, queue: &Queue, clear_type: EClearType) {
        let base_render_pipeline = match &clear_type {
            EClearType::Depth(_) => &self.clear_depth_pipeline,
            EClearType::Color(color) => {
                if color.resolve_target.is_some() {
                    &self.multisample_pipeline
                } else {
                    &self.base_render_pipeline
                }
            }
            EClearType::Both(both) => {
                if both.clear_color.resolve_target.is_some() {
                    &self.multisample_pipeline
                } else {
                    &self.base_render_pipeline
                }
            }
        };

        let color_attachments = match &clear_type {
            EClearType::Depth(_) => vec![],
            EClearType::Color(color) => {
                let color_attachment = if let Some(resolve_target) = color.resolve_target {
                    ColorAttachment {
                        color_ops: Some(Operations {
                            load: LoadOp::Clear(color.color),
                            store: StoreOp::Store,
                        }),
                        view: resolve_target,
                        resolve_target: Some(color.view),
                    }
                } else {
                    ColorAttachment {
                        color_ops: Some(Operations {
                            load: LoadOp::Clear(color.color),
                            store: StoreOp::Store,
                        }),
                        view: color.view,
                        resolve_target: None,
                    }
                };
                vec![color_attachment]
            }
            EClearType::Both(both) => {
                let color_attachment = if let Some(resolve_target) = both.clear_color.resolve_target
                {
                    ColorAttachment {
                        color_ops: Some(Operations {
                            load: LoadOp::Clear(both.clear_color.color),
                            store: StoreOp::Store,
                        }),
                        view: resolve_target,
                        resolve_target: Some(both.clear_color.view),
                    }
                } else {
                    ColorAttachment {
                        color_ops: Some(Operations {
                            load: LoadOp::Clear(both.clear_color.color),
                            store: StoreOp::Store,
                        }),
                        view: both.clear_color.view,
                        resolve_target: None,
                    }
                };
                vec![color_attachment]
            }
        };

        let depth_view = match &clear_type {
            EClearType::Depth(depth) => Some(depth.view),
            EClearType::Color(_) => None,
            EClearType::Both(both) => Some(both.clear_depth.view),
        };

        base_render_pipeline.draw_resources(
            device,
            queue,
            vec![],
            &vec![GpuVertexBufferImp {
                vertex_buffers: &vec![],
                vertex_count: 0,
                index_buffer: None,
                index_count: None,
                draw_type: EDrawCallType::Draw(Draw { instances: 0..1 }),
            }],
            &color_attachments,
            Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            Some(Operations {
                load: LoadOp::Clear(0),
                store: StoreOp::Store,
            }),
            depth_view,
            None,
            None,
        );
    }
}
