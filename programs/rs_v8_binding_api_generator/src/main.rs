use rs_v8_binding_api_generator::{analyzer, engine_api_generator::EngineApiGenerator};

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_module("rs_v8_binding_api_generator", log::LevelFilter::Trace)
        .init();
    let mut analyzer = analyzer::Analyzer::new("rs_engine")?;
    let mut engine_api_generator = EngineApiGenerator::new(
        rs_core_minimal::file_manager::get_engine_output_target_dir()
            .join("generated/rs_v8_engine_binding_api"),
    );
    engine_api_generator.run(&mut analyzer)?;
    Ok(())
}
