use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler_core::pre_process::ShaderDescription;

pub struct ViewDepthShader {}

impl GlobalShader for ViewDepthShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("depth.wgsl"),
            include_dirs: vec![],
            definitions: vec!["PLAYER_VIEW".to_string()],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "ViewDepthShader.wgsl".to_string()
    }
}
