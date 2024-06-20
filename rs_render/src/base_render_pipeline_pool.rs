use crate::base_render_pipeline::BaseRenderPipeline;
use crate::bind_group_layout_entry_hook::EBindGroupLayoutEntryHookType;
use crate::shader_library::ShaderLibrary;
use crate::VertexBufferType;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::{collections::HashMap, hash::Hash};
use wgpu::*;

#[derive(PartialEq, Clone, Default)]
pub struct BaseRenderPipelineBuilder {
    pub shader_name: String,
    pub targets: Vec<Option<ColorTargetState>>,
    pub depth_stencil: Option<DepthStencilState>,
    pub multisample: Option<MultisampleState>,
    pub multiview: Option<NonZeroU32>,
    pub primitive: Option<PrimitiveState>,
    pub vertex_buffer_type: Option<VertexBufferType>,
    pub hooks: Option<HashMap<glam::UVec2, EBindGroupLayoutEntryHookType>>,
    pub vertex_layout_hooks: Option<HashMap<usize, VertexStepMode>>,
}

impl BaseRenderPipelineBuilder {
    pub fn standard(
        texture_format: TextureFormat,
        vertex_buffer_type: Option<VertexBufferType>,
    ) -> BaseRenderPipelineBuilder {
        let mut builder = BaseRenderPipelineBuilder::default();
        builder.targets = vec![Some(ColorTargetState {
            format: texture_format,
            blend: None,
            write_mask: ColorWrites::ALL,
        })];
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Less,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        builder.vertex_buffer_type = vertex_buffer_type;
        builder.primitive = Some(PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        });
        builder
    }

    pub fn standard_alpha_blending(
        texture_format: TextureFormat,
        vertex_buffer_type: Option<VertexBufferType>,
    ) -> BaseRenderPipelineBuilder {
        let mut builder = BaseRenderPipelineBuilder::default();
        builder.targets = vec![Some(ColorTargetState {
            format: texture_format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Less,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        builder.vertex_buffer_type = vertex_buffer_type;
        builder.primitive = Some(PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        });
        builder
    }

    pub fn standard_depth_only(
        vertex_buffer_type: Option<VertexBufferType>,
    ) -> BaseRenderPipelineBuilder {
        let mut builder = BaseRenderPipelineBuilder::default();
        builder.targets = vec![];
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Less,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        builder.vertex_buffer_type = vertex_buffer_type;
        builder.primitive = Some(PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        });
        builder
    }
}

impl Hash for BaseRenderPipelineBuilder {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // self.shader.hash(state);
        self.targets.hash(state);
        self.depth_stencil.hash(state);
        self.multisample.hash(state);
        self.multiview.hash(state);
        self.primitive.hash(state);
        self.vertex_buffer_type.hash(state);
        if let Some(hooks) = &self.hooks {
            let mut keys: Vec<glam::UVec2> = hooks.keys().clone().into_iter().map(|x| *x).collect();
            keys.sort_by(|left, right| {
                if left.y > 1000 || right.y > 1000 {
                    panic!("left.y: {} > 1000 || right.y: {} > 1000", left.y, right.y)
                }
                let l = left.y * 1000000 + left.x;
                let r = right.y * 1000000 + right.x;
                l.cmp(&r)
            });
            for key in keys {
                key.hash(state);
                hooks.get(&key).unwrap().hash(state);
            }
        }
        if let Some(hooks) = &self.vertex_layout_hooks {
            let mut keys: Vec<usize> = hooks.keys().clone().into_iter().map(|x| *x).collect();
            keys.sort();
            for key in keys {
                key.hash(state);
                hooks.get(&key).unwrap().hash(state);
            }
        }
    }
}

impl Eq for BaseRenderPipelineBuilder {}

#[derive(Default)]
pub struct BaseRenderPipelinePool {
    base_render_pipelines: HashMap<BaseRenderPipelineBuilder, Arc<BaseRenderPipeline>>,
}

impl BaseRenderPipelinePool {
    pub fn get(
        &mut self,
        device: &Device,
        shader_library: &ShaderLibrary,
        builder: &BaseRenderPipelineBuilder,
    ) -> Arc<BaseRenderPipeline> {
        if !self.base_render_pipelines.contains_key(builder) {
            let base_render_pipeline =
                BaseRenderPipeline::new(device, shader_library, builder.clone());
            log::trace!("Cache render pipeline: {}", builder.shader_name);
            self.base_render_pipelines
                .insert(builder.clone(), Arc::new(base_render_pipeline));
        }
        self.base_render_pipelines
            .get(builder)
            .expect("Not null")
            .clone()
    }

    pub fn invalid_shader(&mut self, shader_name: impl AsRef<str>) {
        self.base_render_pipelines
            .retain(|k, _| k.shader_name != shader_name.as_ref().to_string());
    }
}
