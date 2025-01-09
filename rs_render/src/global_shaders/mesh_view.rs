use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler_core::pre_process::ShaderDescription;

pub struct MeshViewShader {}

impl GlobalShader for MeshViewShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("mesh_view.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "mesh_view.wgsl".to_string()
    }
}
