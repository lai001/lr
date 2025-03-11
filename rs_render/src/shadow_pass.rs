use crate::{
    base_render_pipeline_pool::BaseRenderPipelinePool,
    global_shaders::{
        depth::{DepthShader, DepthSkinShader},
        global_shader::GlobalShader,
    },
    render_pipeline::generic_pipeline::GenericPipeline,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::{MeshVertex2, MeshVertex5},
    VertexBufferType,
};
use type_layout::TypeLayout;
use wgpu::*;

pub struct ShadowPipelines {
    pub depth_pipeline: GenericPipeline,
    pub depth_skin_pipeline: GenericPipeline,
}

impl ShadowPipelines {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        pool: &mut BaseRenderPipelinePool,
    ) -> ShadowPipelines {
        let depth_pipeline = GenericPipeline::standard_depth_only(
            DepthShader {}.get_name(),
            device,
            shader_library,
            pool,
            Some(VertexBufferType::Interleaved(vec![
                MeshVertex5::type_layout(),
            ])),
        );
        let depth_skin_pipeline = GenericPipeline::standard_depth_only(
            DepthSkinShader {}.get_name(),
            device,
            shader_library,
            pool,
            Some(VertexBufferType::Interleaved(vec![
                MeshVertex5::type_layout(),
                MeshVertex2::type_layout(),
            ])),
        );
        ShadowPipelines {
            depth_pipeline,
            depth_skin_pipeline,
        }
    }
}
