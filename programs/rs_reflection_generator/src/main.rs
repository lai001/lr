use ra_ap_vfs::*;
use rs_reflection_generator::{
    reflection_context::ReflectionContext, token_stream_extension::PrettyPrintStream,
};

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_module("rs_reflection_generator", log::LevelFilter::Trace)
        .init();

    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();

    let workspace_dir = engine_root_dir;

    let manifest_file = AbsPathBuf::assert_utf8(workspace_dir.join("rs_engine/Cargo.toml"));

    let reflection_context = ReflectionContext::new(manifest_file.clone())?;
    let parse_results = reflection_context.parse_crate();

    for parse_result in parse_results {
        let root_dir = AbsPathBuf::assert_utf8(workspace_dir.join("rs_engine")).normalize();
        if let Some(relative_path) = parse_result.file_path.strip_prefix(&root_dir) {
            let output_dir = rs_core_minimal::file_manager::get_engine_generated_dir()
                .join("rs_engine_reflection");
            if !output_dir.exists() {
                std::fs::create_dir_all(&output_dir)?;
            }
            let output_path = output_dir.join(relative_path.as_utf8_path().as_std_path());
            let dir = output_path.parent().expect("A valid path");
            if !dir.exists() {
                let _ = std::fs::create_dir_all(&dir);
            }
            log::debug!("{:?}", &output_path);
            std::fs::write(
                &output_path,
                parse_result
                    .generate_reflection_token_stream()?
                    .to_pretty_string()?,
            )?;
        }
    }
    Ok(())
}
