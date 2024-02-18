use super::global_shader::GlobalShader;
use crate::get_old_buildin_shader_dir;
use rs_shader_compiler::pre_process::ShaderDescription;

pub struct IrradianceCubeMapShader {}

impl GlobalShader for IrradianceCubeMapShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_old_buildin_shader_dir().join("irradiance_cube_map.wgsl"),
            include_dirs: vec![],
            definitions: vec![],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "irradiance_cube_map.wgsl".to_string()
    }
}
