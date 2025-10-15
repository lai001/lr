use clap::{Args, Parser};
use rs_v8_binding_api_generator::{analyzer, engine_api_generator::EngineApiGenerator};

#[derive(Debug, Clone, Args)]
pub struct GeneratorArgs {
    #[arg(short, long)]
    pub manifest_file: std::path::PathBuf,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cli {
    Generator(GeneratorArgs),
}

fn generate(generator_args: GeneratorArgs) -> anyhow::Result<()> {
    let GeneratorArgs { manifest_file } = generator_args;
    let mut analyzer = analyzer::Analyzer::new(&manifest_file)?;
    let mut engine_api_generator = EngineApiGenerator::new();
    engine_api_generator.run(&mut analyzer)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_module(module_path!(), log::LevelFilter::Trace)
        .init();
    let cli = Cli::parse();
    match cli {
        Cli::Generator(generator_args) => generate(generator_args),
    }
}
