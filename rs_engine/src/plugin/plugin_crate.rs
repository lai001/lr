use crate::{
    content::{content_file_type::EContentFileType, level::Level},
    engine::Engine,
    player_viewport::PlayerViewport,
};

pub trait Plugin {
    fn on_init(&mut self, engine: &mut Engine, level: &mut Level, files: &[EContentFileType]);
    fn tick(
        &mut self,
        engine: &mut Engine,
        level: &mut Level,
        ctx: egui::Context,
        player_viewport: &mut PlayerViewport,
        files: &[EContentFileType],
    );
    #[cfg(not(target_os = "android"))]
    fn on_input(&mut self, ty: crate::input_type::EInputType) -> Vec<winit::keyboard::KeyCode>;
}
