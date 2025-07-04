use crate::{
    content::{content_file_type::EContentFileType, level::Level},
    engine::Engine,
    player_viewport::PlayerViewport,
    standalone::application::Application,
};

pub trait Plugin {
    fn on_init(
        &mut self,
        engine: &mut Engine,
        level: &mut Level,
        player_viewport: &mut PlayerViewport,
        files: &[EContentFileType],
    );

    fn tick(
        &mut self,
        engine: &mut Engine,
        ctx: egui::Context,
        files: &[EContentFileType],
        application: &mut Application,
    );

    #[cfg(not(target_os = "android"))]
    fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent);

    #[cfg(not(target_os = "android"))]
    fn on_window_input(
        &mut self,
        window: &mut winit::window::Window,
        ty: crate::input_type::EInputType,
    ) -> Vec<winit::keyboard::KeyCode>;
}
