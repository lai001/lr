pub struct PluginContext {
    pub context: egui::Context,
}

impl PluginContext {
    pub fn new(context: egui::Context) -> Self {
        Self { context }
    }
}
