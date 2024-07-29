fn main() -> anyhow::Result<()> {
    rs_desktop_standalone::application::Application::new()?.run();
    Ok(())
}
