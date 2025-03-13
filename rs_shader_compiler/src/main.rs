use anyhow::anyhow;
use anyhow::Context;
use clap::Parser;
use path_slash::PathBufExt;
use pollster::FutureExt;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    input_file: Option<std::path::PathBuf>,
    #[arg(short, long)]
    definitions: Option<Vec<String>>,
    #[arg(long)]
    include_dirs: Option<Vec<std::path::PathBuf>>,
    #[arg(short, long)]
    output_file: Option<std::path::PathBuf>,
}

fn prepreocess_builtin_shader() -> anyhow::Result<()> {
    let buildin_shaders = rs_render::global_shaders::get_buildin_shaders();
    let output_path: PathBuf = rs_core_minimal::file_manager::get_engine_output_target_dir()
        .canonicalize_slash()?
        .join("shaders")
        .to_slash()
        .ok_or(anyhow!(""))?
        .to_string()
        .into();
    if !output_path.exists() {
        std::fs::create_dir(&output_path)
            .context(anyhow!("Can not create dir {:?}", &output_path))?;
    }

    let mut compile_commands = vec![];
    for buildin_shader in buildin_shaders {
        let description = buildin_shader.get_shader_description();
        let name = buildin_shader.get_name();
        let processed_code = rs_shader_compiler_core::pre_process::pre_process(
            &description.shader_path,
            description.include_dirs.iter(),
            description.definitions.iter(),
        )?;
        let wgsl_filepath = output_path.join(&name);
        match wgsl_filepath.to_slash() {
            Some(filepath) => log::trace!("Writing: {:?}", &filepath),
            None => log::warn!(
                "The path contains non-Unicode sequence: {:?}",
                &wgsl_filepath
            ),
        }
        std::fs::write(&wgsl_filepath, &processed_code)
            .context(anyhow!("Can not write to file {:?}", &wgsl_filepath))?;

        let module = naga::front::wgsl::parse_str(&processed_code)?;
        let bin_data = rs_artifact::bincode_legacy::serialize(&module, None)?;
        let bin_filepath = output_path.join(format!("{}.nagamodule", &name));
        match bin_filepath.to_slash() {
            Some(filepath) => log::trace!("Writing: {:?}", &filepath),
            None => log::warn!(
                "The path contains non-Unicode sequence: {:?}",
                &bin_filepath
            ),
        }
        std::fs::write(&bin_filepath, bin_data)
            .context(anyhow!("Can not write to file {:?}", &bin_filepath))?;

        let compile_command = buildin_shader.as_ref().to_compile_command();
        compile_commands.push(compile_command);
    }
    let output_path = rs_core_minimal::file_manager::get_engine_root_dir().join(".vscode");
    if !output_path.exists() {
        std::fs::create_dir(output_path.clone())
            .context(anyhow!("Can not create dir {:?}", output_path))?;
    }
    let target_path = output_path.join("shader_compile_commands.json");
    std::fs::write(
        target_path.clone(),
        serde_json::to_string(&compile_commands)?,
    )
    .context(anyhow!("Can not write to file {:?}", target_path))?;
    Ok(())
}

fn verify_shaders() -> anyhow::Result<()> {
    let ctx = rs_render::wgpu_context::WGPUContext::windowless(None, None)?;

    let output_path = rs_core_minimal::file_manager::get_engine_output_target_dir().join("shaders");
    for entry in walkdir::WalkDir::new(output_path) {
        let entry = entry?;
        if !entry.path().is_file() {
            continue;
        }
        let path = entry.path();
        let path = std::env::current_dir()?.join(path).canonicalize_slash()?;
        match path.extension() {
            Some(extension) => {
                if extension == "wgsl" {
                    log::trace!("Verifying: {:?}", &path);
                    let shader_source = std::fs::read_to_string(path)?;
                    let module = naga::front::wgsl::parse_str(&shader_source)?;
                    ctx.get_device()
                        .push_error_scope(wgpu::ErrorFilter::Validation);
                    let _ = ctx
                        .get_device()
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: None,
                            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
                        });
                    let err = ctx.get_device().pop_error_scope().block_on();
                    if let Some(err) = err {
                        return Err(anyhow::anyhow!("{}", err));
                    }
                }
            }
            None => {}
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let _ = rs_foundation::change_working_directory();
    let mut builder = env_logger::Builder::new();
    builder.write_style(env_logger::WriteStyle::Auto);
    builder.filter_level(log::LevelFilter::Trace);
    builder.filter_module("naga", log::LevelFilter::Warn);
    builder.filter_module("rs_render", log::LevelFilter::Off);
    builder.filter_module("wgpu_core", log::LevelFilter::Off);
    builder.filter_module("wgpu_hal", log::LevelFilter::Off);
    builder.init();
    let args = Args::try_parse()?;
    match args.input_file {
        Some(input_file) => {
            let result: anyhow::Result<String> = (|| {
                let include_dirs = args.include_dirs.unwrap_or(vec![]);
                let definitions = args.definitions.unwrap_or(vec![]);
                let result = rs_shader_compiler_core::pre_process::pre_process(
                    &input_file,
                    include_dirs.iter(),
                    definitions.iter(),
                )?;
                let _ = naga::front::wgsl::parse_str(&result)?;
                match args.output_file {
                    Some(output_file) => {
                        let _ = std::fs::write(output_file, result.clone())?;
                    }
                    None => {}
                }
                Ok(result)
            })();
            match result {
                Ok(result) => log::trace!("{}", result),
                Err(err) => log::error!("{}", err),
            }
        }
        None => {
            prepreocess_builtin_shader()?;
            verify_shaders()?;
        }
    }
    Ok(())
}
