use anyhow::anyhow;
use clap::{Args, Parser, ValueEnum};
use rs_foundation::change_working_directory;
use rs_media::composition::check_composition;
use std::{fs, path::Path, process};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ELayoutType {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Args)]
struct CompositionArgs {
    #[arg(short, long)]
    input_folder: std::path::PathBuf,
    #[arg(long)]
    file_pattern: String,
    #[arg(long)]
    frames: u32,
    #[arg(long, default_value = "24")]
    frame_rate: u32,
    #[arg(short, long)]
    width: u32,
    #[arg(long)]
    height: u32,
    #[arg(short, long, default_value = "vertical")]
    layout: ELayoutType,
    #[arg(short, long, default_value = "output.mp4")]
    output_file: std::path::PathBuf,
    #[arg(short, long, default_value = "2M")]
    bit_rate: String,
}

#[derive(Debug, Clone, Args)]
struct CheckCompositionArgs {
    #[arg(short, long)]
    input_file: std::path::PathBuf,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
enum Cli {
    Composition(CompositionArgs),
    CheckComposition(CheckCompositionArgs),
}

fn make_composition_video(args: CompositionArgs) -> anyhow::Result<()> {
    log::trace!("{args:?}");
    if Path::new("./intermediate").exists() {
        fs::remove_dir_all(Path::new("./intermediate"))?;
    }
    fs::create_dir_all("./intermediate/alpha")?;
    fs::create_dir_all("./intermediate/background")?;

    let alpha_overlay_x = match args.layout {
        ELayoutType::Vertical => 0,
        ELayoutType::Horizontal => args.width,
    };
    let alpha_overlay_y = match args.layout {
        ELayoutType::Vertical => args.height,
        ELayoutType::Horizontal => 0,
    };

    let output = process::Command::new("ffmpeg")
        .arg("-i")
        .arg(args.input_folder.join(&args.file_pattern))
        .arg("-vf")
        .arg("alphaextract,format=rgba")
        .arg("-y")
        .arg(format!("./intermediate/alpha/{}", &args.file_pattern))
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow!(stderr));
    }

    let output = process::Command::new("ffmpeg")
        .arg("-f")
        .arg("lavfi")
        .arg("-i")
        .arg(format!(
            "color=black:{}x{}:d=3,format=rgba",
            match args.layout {
                ELayoutType::Vertical => args.width,
                ELayoutType::Horizontal => args.width * 2,
            },
            match args.layout {
                ELayoutType::Vertical => args.height * 2,
                ELayoutType::Horizontal => args.height,
            }
        ))
        .arg("-frames:v")
        .arg(format!("{}", args.frames))
        .arg("-y")
        .arg(format!("./intermediate/background/{}", &args.file_pattern))
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow!(stderr));
    }

    let mut ffmpeg_cmd = process::Command::new("ffmpeg");
    ffmpeg_cmd
        .arg("-i")
        .arg(format!("./intermediate/background/{}", &args.file_pattern))
        .arg("-framerate")
        .arg(format!("{}", args.frame_rate))
        //
        .arg("-i")
        .arg(args.input_folder.join(&args.file_pattern))
        .arg("-framerate")
        .arg(format!("{}", args.frame_rate))
        //
        .arg("-i")
        .arg(format!("./intermediate/alpha/{}", &args.file_pattern))
        .arg("-framerate")
        .arg(format!("{}", args.frame_rate))
        //
        .arg("-filter_complex")
        .arg(format!(
            "[0][1]overlay=0:0,format=rgba[bg];[bg][2]overlay={}:{},format=yuv420p[v]",
            alpha_overlay_x, alpha_overlay_y
        ))
        .arg("-map")
        .arg("[v]")
        .arg("-g")
        .arg("1")
        .arg("-b:v")
        .arg(args.bit_rate)
        .arg("-y")
        .arg(format!("./intermediate/input.mp4"));

    let output = ffmpeg_cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow!(stderr));
    }

    let output = process::Command::new("ffmpeg")
        .arg("-i")
        .arg(format!("./intermediate/input.mp4"))
        .arg("-c")
        .arg("copy")
        .arg("-metadata")
        .arg(format!(
            r#"composition_info="0,0,{w},{h};{x},{y},{w},{h}""#,
            w = args.width,
            h = args.height,
            x = alpha_overlay_x,
            y = alpha_overlay_y
        ))
        .arg("-movflags")
        .arg("+use_metadata_tags")
        .arg("-y")
        .arg(args.output_file)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow!(stderr));
    }

    Ok(())
}

fn main() {
    change_working_directory();
    let mut builder = env_logger::Builder::new();
    builder.write_style(env_logger::WriteStyle::Auto);
    builder.filter_level(log::LevelFilter::Trace);
    builder.init();

    match Cli::parse() {
        Cli::Composition(args) => {
            make_composition_video(args).unwrap();
        }
        Cli::CheckComposition(args) => {
            let composition = check_composition(args.input_file).unwrap();
            log::trace!("{:?}", composition);
        }
    }
}
