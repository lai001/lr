use rs_shader_compiler::pre_process::ShaderDescription;

pub trait GlobalShader {
    fn get_shader_description(&self) -> ShaderDescription;
    fn get_name(&self) -> String;
}
