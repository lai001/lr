use anyhow::anyhow;
use anyhow::Context;
use clap::Parser;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;

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
    let output_path = rs_core_minimal::file_manager::get_engine_output_target_dir().join("shaders");
    if !output_path.exists() {
        std::fs::create_dir(output_path.clone())
            .context(anyhow!("Can not create dir {:?}", output_path))?;
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
        let filepath = output_path.join(name);
        std::fs::write(filepath.clone(), processed_code)
            .context(anyhow!("Can not write to file {:?}", filepath))?;
        match filepath.canonicalize_slash() {
            Ok(filepath) => {
                log::trace!("Writing: {:?}", &filepath);
            }
            Err(err) => {
                log::warn!("{}", err);
            }
        }
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
    let output_path = rs_core_minimal::file_manager::get_engine_output_target_dir().join("shaders");
    for entry in walkdir::WalkDir::new(output_path) {
        let entry = entry?;
        if !entry.path().is_file() {
            continue;
        }
        let path = entry.path();
        let path = std::env::current_dir()?.join(path).canonicalize_slash()?;
        log::trace!("Verifying: {:?}", &path);
        let shader_source = std::fs::read_to_string(path)?;
        naga::front::wgsl::parse_str(&shader_source)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.write_style(env_logger::WriteStyle::Auto);
    builder.filter_level(log::LevelFilter::Trace);
    builder.filter_module("naga", log::LevelFilter::Warn);
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
