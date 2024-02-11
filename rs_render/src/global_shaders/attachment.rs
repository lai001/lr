use super::global_shader::GlobalShader;
use crate::get_cargo_manifest_dir;
use rs_shader_compiler::pre_process::ShaderDescription;

pub struct AttachmentShader {}

impl GlobalShader for AttachmentShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_cargo_manifest_dir()
                .join("../rs_computer_graphics/src/shader/attachment.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "attachment.wgsl".to_string()
    }
}
