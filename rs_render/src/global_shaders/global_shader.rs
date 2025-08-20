use rs_shader_compiler_core::pre_process::ShaderDescription;
#[cfg(feature = "editor")]
mod editor_mod {
    pub use path_slash::PathBufExt;
    pub use rs_core_minimal::path_ext::CanonicalizeSlashExt;
    pub use rs_shader_compiler_core::compile_command::CompileCommand;
}
#[cfg(feature = "editor")]
use editor_mod::*;

pub trait GlobalShader {
    fn get_shader_description(&self) -> ShaderDescription;
    fn get_name(&self) -> String;
    fn is_support_features(&self, features: &wgpu::Features) -> bool {
        let _ = features;
        true
    }
    fn is_support_limits(&self, limits: &wgpu::Limits) -> bool {
        let _ = limits;
        true
    }
    #[cfg(feature = "editor")]
    fn to_compile_command(&self) -> CompileCommand {
        let shader_description = self.get_shader_description();
        let mut arguments = vec![];
        for include_dir in &shader_description.include_dirs {
            let include_dir = include_dir.canonicalize_slash().unwrap();
            let include_dir = include_dir.to_slash_lossy();
            arguments.push(format!("-I{}", include_dir));
        }
        for definition in &shader_description.definitions {
            arguments.push(format!("-D{definition}"));
        }
        CompileCommand {
            arguments,
            file: shader_description
                .clone()
                .shader_path
                .canonicalize_slash()
                .unwrap()
                .to_slash_lossy()
                .to_string(),
        }
    }
}
