use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler::pre_process::ShaderDescription;

pub struct LightCullingShader {}

impl GlobalShader for LightCullingShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("light_culling.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "light_culling.wgsl".to_string()
    }
}
