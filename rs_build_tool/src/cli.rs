use clap::{Args, Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum ModeType {
    #[default]
    Editor,
    Standalone,
}

#[derive(Debug, Clone, ValueEnum, Default, Eq, PartialEq)]
pub enum ProfileType {
    #[default]
    Release,
    Debug,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectFilesArgs {
    #[arg(short, long)]
    pub project_file: std::path::PathBuf,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectArgs {
    #[arg(short, long, default_value_t = false)]
    pub is_enable: bool,
    #[arg(short, long)]
    pub project_file: std::path::PathBuf,
    #[arg(short, long)]
    pub mode_type: ModeType,
    #[arg(short, long)]
    pub profile_type: ProfileType,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cli {
    ProjectFiles(ProjectFilesArgs),
    Project(ProjectArgs),
    CreateDefaultLoadPluginsFile,
}
