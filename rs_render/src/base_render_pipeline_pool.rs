use crate::base_render_pipeline::BaseRenderPipeline;
use crate::bind_group_layout_entry_hook::EBindGroupLayoutEntryHookType;
use crate::global_shaders::global_shader::GlobalShader;
use crate::shader_library::ShaderLibrary;
use crate::VertexBufferType;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::{collections::HashMap, hash::Hash};
use wgpu::{ColorTargetState, DepthStencilState, Device, MultisampleState, PrimitiveState};

#[derive(PartialEq, Clone, Default)]
pub struct BaseRenderPipelineBuilder {
    // shader: ShaderDescription,
    targets: Vec<Option<ColorTargetState>>,
    depth_stencil: Option<DepthStencilState>,
    multisample: Option<MultisampleState>,
    multiview: Option<NonZeroU32>,
    primitive: Option<PrimitiveState>,
    vertex_buffer_type: Option<VertexBufferType>,
    hooks: Option<HashMap<glam::UVec2, EBindGroupLayoutEntryHookType>>,
}

impl BaseRenderPipelineBuilder {
    pub fn set_targets(mut self, targets: Vec<Option<ColorTargetState>>) -> Self {
        self.targets = targets;
        self
    }

    pub fn set_depth_stencil(mut self, depth_stencil: Option<DepthStencilState>) -> Self {
        self.depth_stencil = depth_stencil;
        self
    }

    pub fn set_multisample(mut self, multisample: Option<MultisampleState>) -> Self {
        self.multisample = multisample;
        self
    }

    pub fn set_multiview(mut self, multiview: Option<NonZeroU32>) -> Self {
        self.multiview = multiview;
        self
    }

    pub fn set_primitive(mut self, primitive: Option<PrimitiveState>) -> Self {
        self.primitive = primitive;
        self
    }

    pub fn set_vertex_buffer_type(mut self, vertex_buffer_type: Option<VertexBufferType>) -> Self {
        self.vertex_buffer_type = vertex_buffer_type;
        self
    }

    pub fn set_hooks(
        mut self,
        hooks: Option<HashMap<glam::UVec2, EBindGroupLayoutEntryHookType>>,
    ) -> Self {
        self.hooks = hooks;
        self
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
        global_shader: &impl GlobalShader,
        builder: &BaseRenderPipelineBuilder,
    ) -> Arc<BaseRenderPipeline> {
        if !self.base_render_pipelines.contains_key(builder) {
            let base_render_pipeline = BaseRenderPipeline::new(
                device,
                shader_library,
                global_shader,
                &builder.targets,
                builder.depth_stencil.clone(),
                builder.multisample,
                builder.multiview,
                builder.primitive,
                builder.vertex_buffer_type.clone(),
                builder.hooks.clone(),
            );
            self.base_render_pipelines
                .insert(builder.clone(), Arc::new(base_render_pipeline));
        }
        self.base_render_pipelines
            .get(builder)
            .expect("Not null")
            .clone()
    }
}
