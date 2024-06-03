use crate::{
    base_render_pipeline_pool::BaseRenderPipelinePool,
    global_shaders::{
        depth::{DepthShader, DepthSkinShader},
        global_shader::GlobalShader,
    },
    render_pipeline::generic_pipeline::GenericPipeline,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::{MeshVertex0, MeshVertex2},
    VertexBufferType,
};
use type_layout::TypeLayout;
use wgpu::*;

pub struct ShadowPipilines {
    pub depth_pipeline: GenericPipeline,
    pub depth_skin_pipeline: GenericPipeline,
}

impl ShadowPipilines {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        pool: &mut BaseRenderPipelinePool,
    ) -> ShadowPipilines {
        let depth_pipeline = GenericPipeline::standard_depth_only(
            DepthShader {}.get_name(),
            device,
            shader_library,
            pool,
            Some(VertexBufferType::Interleaved(vec![
                MeshVertex0::type_layout(),
            ])),
        );
        let depth_skin_pipeline = GenericPipeline::standard_depth_only(
            DepthSkinShader {}.get_name(),
            device,
            shader_library,
            pool,
            Some(VertexBufferType::Interleaved(vec![
                MeshVertex0::type_layout(),
                MeshVertex2::type_layout(),
            ])),
        );
        ShadowPipilines {
            depth_pipeline,
            depth_skin_pipeline,
        }
    }
}
