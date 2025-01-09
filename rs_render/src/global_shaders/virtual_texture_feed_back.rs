use super::{global_shader::GlobalShader, skeleton_shading::NUM_MAX_BONE};
use crate::get_buildin_shader_dir;
use rs_shader_compiler_core::pre_process::ShaderDescription;

pub struct SkinMeshVirtualTextureFeedBackShader {}

impl GlobalShader for SkinMeshVirtualTextureFeedBackShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("virtual_texture_feed_back.wgsl"),
            include_dirs: vec![],
            definitions: vec![format!("SKELETON_MAX_BONES={NUM_MAX_BONE}")],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "virtual_texture_feed_back.wgsl".to_string()
    }
}

pub struct StaticMeshVirtualTextureFeedBackShader {}

impl GlobalShader for StaticMeshVirtualTextureFeedBackShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("virtual_texture_feed_back.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "static_mesh_virtual_texture_feed_back.wgsl".to_string()
    }
}
