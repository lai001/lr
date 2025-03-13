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

    let mut reflection_context = ReflectionContext::new(manifest_file.clone())?;
    // reflection_context.dump_all_files();

    let parse_file_result = reflection_context.parse_file(AbsPathBuf::assert_utf8(
        workspace_dir.join("rs_engine/src/engine.rs"),
    ))?;

    let output_dir =
        rs_core_minimal::file_manager::get_engine_generated_dir().join("rs_engine_reflection");
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }
    let output_path = output_dir.join("engine_reflection.rs");
    std::fs::write(
        output_path,
        parse_file_result
            .generate_reflection_token_stream()?
            .to_pretty_string()?,
    )?;

    Ok(())
}
