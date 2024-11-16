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

    std::fs::write(
        workspace_dir.join("rs_reflection_generator/target/engine_reflection.rs"),
        parse_file_result
            .generate_reflection_token_stream()?
            .to_pretty_string()?,
    )?;

    Ok(())
}
