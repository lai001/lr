use crate::base_compute_pipeline::BaseComputePipeline;
use crate::shader_library::ShaderLibrary;
use std::cell::RefCell;
use std::sync::Arc;
use std::{collections::HashMap, hash::Hash};
use wgpu::*;

#[derive(PartialEq, Clone, Default)]
pub struct BaseComputePipelineBuilder {
    pub shader_name: String,
}

impl BaseComputePipelineBuilder {
    pub fn new(shader_name: String) -> BaseComputePipelineBuilder {
        BaseComputePipelineBuilder { shader_name }
    }
}

impl Hash for BaseComputePipelineBuilder {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.shader_name.hash(state);
    }
}

impl Eq for BaseComputePipelineBuilder {}

#[derive(Default)]
pub struct BaseComputePipelinePool {
    compute_pipelines: RefCell<HashMap<BaseComputePipelineBuilder, Arc<BaseComputePipeline>>>,
}

impl BaseComputePipelinePool {
    pub fn get(
        &self,
        device: &Device,
        shader_library: &ShaderLibrary,
        builder: &BaseComputePipelineBuilder,
    ) -> Arc<BaseComputePipeline> {
        let mut compute_pipelines = self.compute_pipelines.borrow_mut();
        let entry = compute_pipelines.entry(builder.clone());
        entry
            .or_insert_with(|| {
                let pipeline =
                    BaseComputePipeline::new(device, shader_library, &builder.shader_name);
                Arc::new(pipeline)
            })
            .clone()
    }

    pub fn invalid_shader(&self, shader_name: impl AsRef<str>) {
        let mut compute_pipelines = self.compute_pipelines.borrow_mut();
        compute_pipelines.retain(|k, _| k.shader_name != shader_name.as_ref().to_string());
    }
}
