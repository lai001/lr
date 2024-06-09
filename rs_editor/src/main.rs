fn main() -> anyhow::Result<()> {
    rs_editor::editor::Editor::new().run()?;
    Ok(())
}
