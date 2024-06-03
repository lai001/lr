use crate::{
    base_compute_pipeline::BaseComputePipeline,
    base_compute_pipeline_pool::{BaseComputePipelineBuilder, BaseComputePipelinePool},
    global_shaders::{
        format_conversion::Depth32FloatConvertRGBA8UnormShader, global_shader::GlobalShader,
    },
    shader_library::ShaderLibrary,
};
use std::sync::Arc;
use wgpu::*;

pub struct Depth32FloatConvertRGBA8UnormPipeline {
    base_compute_pipeline: Arc<BaseComputePipeline>,
}

impl Depth32FloatConvertRGBA8UnormPipeline {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        pool: &BaseComputePipelinePool,
    ) -> Depth32FloatConvertRGBA8UnormPipeline {
        let base_compute_pipeline = pool.get(
            device,
            shader_library,
            &BaseComputePipelineBuilder {
                shader_name: Depth32FloatConvertRGBA8UnormShader {}.get_name(),
            },
        );
        Depth32FloatConvertRGBA8UnormPipeline {
            base_compute_pipeline,
        }
    }

    pub fn execute(
        &self,
        device: &Device,
        queue: &Queue,
        input_texture_view: &TextureView,
        output_texture_view: &TextureView,
        workgroups: glam::UVec3,
    ) {
        self.base_compute_pipeline.execute_resources(
            device,
            queue,
            vec![vec![
                BindingResource::TextureView(&input_texture_view),
                BindingResource::TextureView(&output_texture_view),
            ]],
            workgroups,
        );
    }
}
