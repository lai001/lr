use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler_core::pre_process::ShaderDescription;

pub struct PanoramaToCubeShader {}

impl GlobalShader for PanoramaToCubeShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("panorama_to_cube.wgsl"),
            include_dirs: vec![],
            definitions: vec![Self::get_definition()],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "panorama_to_cube.wgsl".to_string()
    }
}

impl PanoramaToCubeShader {
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
