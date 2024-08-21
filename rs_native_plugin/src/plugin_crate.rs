use rs_engine::engine::Engine;

pub trait Plugin {
    fn tick(&mut self, engine: &mut Engine, ctx: egui::Context);
}
