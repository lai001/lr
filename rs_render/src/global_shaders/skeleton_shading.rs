use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler::pre_process::{Definition, ShaderDescription};

pub struct SkeletonShadingShader {}

impl GlobalShader for SkeletonShadingShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("phong_shading.wgsl"),
            include_dirs: vec![],
            definitions: vec![Definition {
                name: "SKELETON_MAX_BONES".to_string(),
                value: Some("255".to_string()),
            }],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "phong_skeleton_shading.wgsl".to_string()
    }
}
