use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler::pre_process::ShaderDescription;

pub struct JFACompositionShader {}

impl GlobalShader for JFACompositionShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("jfa_composition.wgsl"),
            include_dirs: vec![],
            definitions: vec!["USE_GRAYSCALE".to_string()],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "jfa_composition.wgsl".to_string()
    }
}
