use crate::{
    base_render_pipeline::BaseRenderPipeline,
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    shader_library::ShaderLibrary,
    VertexBufferType,
};
use std::sync::Arc;
use wgpu::*;

pub struct GenericPipeline {
    pub builder: BaseRenderPipelineBuilder,
    pub base_render_pipeline: Arc<BaseRenderPipeline>,
}

impl GenericPipeline {
    pub fn standard(
        shader_name: String,
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
        vertex_buffer_type: Option<VertexBufferType>,
    ) -> GenericPipeline {
        let mut builder = BaseRenderPipelineBuilder::standard(*texture_format, vertex_buffer_type);
        builder.shader_name = shader_name;

        let base_render_pipeline = pool.get(device, shader_library, &builder);
        GenericPipeline {
            builder,
            base_render_pipeline,
        }
    }

    pub fn standard_alpha_blending(
        shader_name: String,
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
        vertex_buffer_type: Option<VertexBufferType>,
    ) -> GenericPipeline {
        let mut builder =
            BaseRenderPipelineBuilder::standard_alpha_blending(*texture_format, vertex_buffer_type);
        builder.shader_name = shader_name;
        let base_render_pipeline = pool.get(device, shader_library, &builder);
        GenericPipeline {
            builder,
            base_render_pipeline,
        }
    }

    pub fn standard_depth_only(
        shader_name: String,
        device: &Device,
        shader_library: &ShaderLibrary,
        pool: &mut BaseRenderPipelinePool,
        vertex_buffer_type: Option<VertexBufferType>,
    ) -> GenericPipeline {
        let mut builder = BaseRenderPipelineBuilder::standard_depth_only(vertex_buffer_type);
        builder.shader_name = shader_name;

        let base_render_pipeline = pool.get(device, shader_library, &builder);
        GenericPipeline {
            builder,
            base_render_pipeline,
        }
    }
}
