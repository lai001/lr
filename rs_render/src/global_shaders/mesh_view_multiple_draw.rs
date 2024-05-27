use super::global_shader::GlobalShader;
use crate::get_buildin_shader_dir;
use rs_shader_compiler::pre_process::ShaderDescription;

pub struct MeshViewMultipleDrawShader {}

impl GlobalShader for MeshViewMultipleDrawShader {
    fn get_shader_description(&self) -> ShaderDescription {
        let shader_description = ShaderDescription {
            shader_path: get_buildin_shader_dir().join("mesh_view.wgsl"),
            include_dirs: vec![],
            definitions: vec![format!("MULTIPLE_DRAW=")],
        };
        shader_description
    }

    fn get_name(&self) -> String {
        "mesh_view_multiple_draw.wgsl".to_string()
    }
}
