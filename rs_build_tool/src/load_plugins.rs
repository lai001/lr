pub fn create_load_plugins_file(
    crate_name: &str,
    plugin_name: Option<String>,
    is_overwrite: bool,
) -> anyhow::Result<()> {
    let engine_output_target_dir = rs_core_minimal::file_manager::get_engine_output_target_dir();
    let load_plugins_file_dir = engine_output_target_dir.join(format!("generated/{}", crate_name));
    if !load_plugins_file_dir.exists() {
        std::fs::create_dir_all(&load_plugins_file_dir)?;
    }
    let file_name = "load_plugins.generated.rs";
    let contents = create_load_plugins_source(plugin_name);
    let target_file_path = load_plugins_file_dir.join(&file_name);
    if !target_file_path.exists()
        || std::fs::read_to_string(&target_file_path)?.is_empty()
        || is_overwrite
    {
        std::fs::write(&target_file_path, contents)?;
    }
    Ok(())
}

fn create_load_plugins_source(plugin_name: Option<String>) -> String {
    let template = r#"struct LoadPlugins {}

impl LoadPlugins {
    fn load() -> Vec<Box<dyn rs_engine::plugin::plugin_crate::Plugin>> {
        @plugins@
    }
}
"#;
    match plugin_name {
        Some(plugin_name) => template.replace(
            "@plugins@",
            &format!("vec![{}::create_plugin()]", plugin_name),
        ),
        None => template.replace("@plugins@", "vec![]"),
    }
}
