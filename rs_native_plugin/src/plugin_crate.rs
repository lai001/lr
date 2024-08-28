use rs_engine::{content::level::Level, engine::Engine, player_viewport::PlayerViewport};

#[derive(Clone)]
pub enum EInputType<'a> {
    Device(&'a winit::event::DeviceEvent),
    MouseWheel(&'a winit::event::MouseScrollDelta),
    MouseInput(
        &'a winit::event::ElementState,
        &'a winit::event::MouseButton,
    ),
    KeyboardInput(
        &'a std::collections::HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
    ),
}

pub trait Plugin {
    fn on_init(&mut self, engine: &mut Engine, level: &mut Level);
    fn tick(
        &mut self,
        engine: &mut Engine,
        level: &mut Level,
        ctx: egui::Context,
        player_viewport: &mut PlayerViewport,
    );
    #[cfg(not(target_os = "android"))]
    fn on_input(&mut self, ty: EInputType);
}
