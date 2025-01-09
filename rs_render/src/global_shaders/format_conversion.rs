use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler_core::pre_process::ShaderDescription;

pub struct Depth32FloatConvertRGBA8UnormShader {}

impl GlobalShader for Depth32FloatConvertRGBA8UnormShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("format_conversion.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "Depth32FloatConvertRGBA8Unorm.wgsl".to_string()
    }
}
