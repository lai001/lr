use clap::{Args, Parser};

#[derive(Debug, Clone, Args)]
pub struct ProjectFilesArgs {
    #[arg(short, long)]
    pub project_file: std::path::PathBuf,
}

#[derive(Debug, Clone, Args)]
pub struct HotreloadArgs {
    #[arg(short, long, default_value_t = false)]
    pub is_enable: bool,
    #[arg(short, long)]
    pub project_file: std::path::PathBuf,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cli {
    ProjectFiles(ProjectFilesArgs),
    Hotreload(HotreloadArgs),
}
