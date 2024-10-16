fn main() -> anyhow::Result<()> {
    rs_editor::editor::Editor::new().run()?;
    #[cfg(feature = "exit_check")]
    let _ = std::io::stdin().read_line(&mut String::new());
    Ok(())
}
