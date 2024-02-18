use super::global_shader::GlobalShader;
use crate::get_old_buildin_shader_dir;
use rs_shader_compiler::pre_process::ShaderDescription;

pub struct VirtualTextureFeedBackShader {}

impl GlobalShader for VirtualTextureFeedBackShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_old_buildin_shader_dir().join("virtual_texture_feed_back.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "virtual_texture_feed_back.wgsl".to_string()
    }
}
