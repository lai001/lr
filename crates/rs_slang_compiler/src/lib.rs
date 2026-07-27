pub mod error;
pub mod intellisense_support;

use rs_foundation::full_cmd_from_command;
use semver::Version;
use std::{
    collections::{HashMap, HashSet},
    env::{self, consts::EXE_SUFFIX},
    path::PathBuf,
    process::Command,
};

fn find_slangc() -> Option<PathBuf> {
    let key = "PATH";
    let mut found_paths: HashMap<Version, PathBuf> = HashMap::new();
    match env::var_os(key) {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                let executable_name = format!("slangc{}", EXE_SUFFIX);
                let path = path.join(executable_name);
                if path.exists() {
                    let output = Command::new(&path).arg("-version").output();
                    match output {
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let version = Version::parse(&format!("{}.0", stderr.trim()));
                            if let Ok(version) = version {
                                found_paths.insert(version, path);
                            }
                        }
                        Err(err) => {
                            let _ = err;
                        }
                    };
                }
            }
        }
        None => {
            return None;
        }
    }

    let versions = found_paths.keys().collect::<Vec<&Version>>();
    let max_version = versions.iter().max_by_key(|lhs| *lhs);
    if let Some(max_version) = max_version
        && let Some(found_path) = found_paths.get(*max_version)
    {
        return Some(found_path.to_path_buf());
    }

    return None;
}

#[derive(Default, Debug, Clone)]
pub struct CompileOptions {
    pub includes: HashSet<PathBuf>,
    pub definitions: HashSet<String>,
    pub is_debug: bool,
    pub preserve_params: bool,
    pub obfuscate: bool,
    pub no_mangle: bool,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

impl CompileOptions {
    pub fn add_define(&mut self, definition: String) {
        self.definitions.insert(definition);
    }

    pub fn add_include<P: Into<PathBuf>>(&mut self, path: P) {
        self.includes.insert(path.into());
    }
}

pub fn compile(
    compile_options: &CompileOptions,
    out_command: Option<&mut String>,
) -> crate::error::Result<String> {
    let path = find_slangc().ok_or(crate::error::Error::Io(std::io::ErrorKind::NotFound.into()))?;
    let mut command = Command::new(&path);

    let CompileOptions {
        includes,
        definitions,
        is_debug,
        input_path,
        output_path,
        preserve_params,
        obfuscate,
        no_mangle,
    } = compile_options;
    if !input_path.exists() {
        return Err(crate::error::Error::Io(std::io::ErrorKind::NotFound.into()));
    }

    for include in includes {
        command.arg(format!("-I{}", include.display()));
    }
    for definition in definitions {
        command.arg(format!("-D{}", definition));
    }
    if *is_debug {
        command.arg("-g3");
    }
    if *preserve_params {
        command.arg("-preserve-params");
    }
    if *obfuscate {
        command.arg("-obfuscate");
    }
    if *no_mangle {
        command.arg("-no-mangle");
    }

    command.arg(input_path);

    std::fs::create_dir_all(
        output_path
            .parent()
            .ok_or(crate::error::Error::Io(std::io::ErrorKind::NotFound.into()))?,
    )?;

    command.arg("-o");
    command.arg(output_path);

    let output = command.output()?;

    if let Some(out_command) = out_command {
        let cmd = full_cmd_from_command(&command);
        *out_command = cmd;
    }

    if output.status.success() && output.stderr.is_empty() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    } else {
        let message = format!("{}", String::from_utf8_lossy(&output.stderr));
        return Err(crate::error::Error::Compilation(message));
    }
}

#[cfg(test)]
mod tests {
    use crate::{CompileOptions, compile};
    use eolify::{CRLF, Normalize};

    fn expected() -> &'static str {
        r#"struct VertexOutput_0
{
    @builtin(position) position_0 : vec4<f32>,
};

struct vertexInput_0
{
    @location(0) position_1 : vec3<f32>,
};

@vertex
fn vs_main( _S1 : vertexInput_0, @builtin(vertex_index) vertexID_0 : u32) -> VertexOutput_0
{
    var vertex_output_0 : VertexOutput_0;
    vertex_output_0.position_0 = vec4<f32>(0.0f);
    return vertex_output_0;
}

struct FragmentOutput_0
{
    @location(0) color_0 : vec4<f32>,
};

@fragment
fn fs_main(@builtin(position) position_2 : vec4<f32>) -> FragmentOutput_0
{
    var fragment_output_0 : FragmentOutput_0;
    fragment_output_0.color_0 = vec4<f32>(0.0f);
    return fragment_output_0;
}

"#
    }

    #[test]
    fn test_compile() {
        let root =
            rs_core_minimal::file_manager::get_engine_build_tmp_dir().join("rs_slang_compiler");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::create_dir_all(&root);
        let root = root.canonicalize().unwrap();

        let contents = r#"
struct VertexIn
{
    [[vk::location(0)]] float3 position;
};
struct VertexOutput
{
    float4 position : SV_Position;
};
struct FragmentOutput
{
    [[vk::location(0)]] float4 color : SV_Target0;
};
[shader("vertex")]
VertexOutput vs_main(uint vertexID: SV_VertexID, VertexIn vertex_in)
{
    VertexOutput vertex_output;
    vertex_output.position = float4(0.0);
    return vertex_output;
}

[shader("fragment")]
FragmentOutput fs_main(VertexOutput vertex_output)
{
    FragmentOutput fragment_output;
    fragment_output.color = float4(0.0);
    return fragment_output;
}"#;
        let path = root.join("test.slang");
        std::fs::write(&path, contents).unwrap();
        let mut options = CompileOptions::default();
        options.input_path = path;
        options.output_path = root.join("out").join("test.wgsl");
        let _ = compile(&options, None)
            .map_err(|err| err.to_string())
            .unwrap();
        let shader_source = std::fs::read_to_string(&options.output_path).unwrap();
        assert_eq!(
            CRLF::normalize_str(&shader_source),
            CRLF::normalize_str(expected())
        );

        let ctx = rs_render_core::wgpu_context::WGPUContext::windowless(None, None, None).unwrap();
        let module = naga::front::wgsl::parse_str(&shader_source).unwrap();
        let error_scope = ctx
            .get_device()
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let _ = ctx
            .get_device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
            });
        let err = pollster::FutureExt::block_on(error_scope.pop());
        assert!(err.is_none());
    }
}
