use super::global_shader::GlobalShader;
use crate::get_cargo_manifest_dir;
use rs_shader_compiler::pre_process::ShaderDescription;

pub struct PanoramaToCubeShader {}

impl GlobalShader for PanoramaToCubeShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_cargo_manifest_dir()
                .join("../rs_computer_graphics/src/shader/panorama_to_cube.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "panorama_to_cube.wgsl".to_string()
    }
}
