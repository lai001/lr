use path_slash::PathBufExt;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use rs_shader_compiler::{compile_command::CompileCommand, pre_process::ShaderDescription};

pub trait GlobalShader {
    fn get_shader_description(&self) -> ShaderDescription;
    fn get_name(&self) -> String;

    #[cfg(feature = "editor")]
    fn to_compile_command(&self) -> CompileCommand {
        let shader_description = self.get_shader_description();
        let mut arguments = vec![];
        for include_dir in &shader_description.include_dirs {
            let include_dir = include_dir.canonicalize_slash().unwrap().to_slash_lossy();
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
                .to_slash_lossy(),
        }
    }
}
