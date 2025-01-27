use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler_core::pre_process::ShaderDescription;

pub struct PrimitiveShader {}

impl GlobalShader for PrimitiveShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("primitive.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "PrimitiveShader.wgsl".to_string()
    }
}
