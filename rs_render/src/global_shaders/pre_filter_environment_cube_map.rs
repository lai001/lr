use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler::pre_process::{Definition, ShaderDescription};

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

    pub fn get_definition() -> Definition {
        match Self::get_format() {
            wgpu::TextureFormat::Rg16Float => Definition {
                name: "TEXTURE_FORMAT".to_string(),
                value: Some("rg16float".to_string()),
            },
            wgpu::TextureFormat::Rg32Float => Definition {
                name: "TEXTURE_FORMAT".to_string(),
                value: Some("rg32float".to_string()),
            },
            wgpu::TextureFormat::Rgba16Float => Definition {
                name: "TEXTURE_FORMAT".to_string(),
                value: Some("rgba16float".to_string()),
            },
            wgpu::TextureFormat::Rgba32Float => Definition {
                name: "TEXTURE_FORMAT".to_string(),
                value: Some("rgba32float".to_string()),
            },
            _ => panic!(),
        }
    }
}
