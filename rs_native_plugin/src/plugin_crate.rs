use rs_engine::{content::level::Level, engine::Engine};

pub trait Plugin {
    fn on_init(&mut self, engine: &mut Engine, level: &mut Level);
    fn tick(&mut self, engine: &mut Engine, level: &mut Level, ctx: egui::Context);
}
