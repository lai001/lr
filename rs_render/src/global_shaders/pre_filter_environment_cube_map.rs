use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler::pre_process::ShaderDescription;

pub struct PreFilterEnvironmentCubeMapShader {}

impl GlobalShader for PreFilterEnvironmentCubeMapShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("pre_filter_environment_cube_map.wgsl"),
            include_dirs: vec![],
            definitions: vec![Self::get_definition()],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "pre_filter_environment_cube_map.wgsl".to_string()
    }
}

impl PreFilterEnvironmentCubeMapShader {
    pub fn get_format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba32Float
    }

    pub fn get_definition() -> String {
        match Self::get_format() {
            wgpu::TextureFormat::Rg16Float => "TEXTURE_FORMAT=rg16float".to_string(),
            wgpu::TextureFormat::Rg32Float => "TEXTURE_FORMAT=rg32float".to_string(),
            wgpu::TextureFormat::Rgba16Float => "TEXTURE_FORMAT=rgba16float".to_string(),
            wgpu::TextureFormat::Rgba32Float => "TEXTURE_FORMAT=rgba32float".to_string(),
            _ => panic!(),
        }
    }
}
