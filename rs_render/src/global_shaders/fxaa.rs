use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler_core::pre_process::ShaderDescription;

pub struct FXAAShader {}

impl GlobalShader for FXAAShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("fxaa.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "FXAAShader.wgsl".to_string()
    }
}
