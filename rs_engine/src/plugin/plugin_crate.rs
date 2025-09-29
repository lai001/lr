use crate::{
    content::{content_file_type::EContentFileType, level::Level},
    engine::Engine,
    standalone::application::Application,
};

pub trait Plugin {
    fn on_init(
        &mut self,
        engine: &mut Engine,
        level: &mut Level,
        application: &mut Application,
        contents: &[EContentFileType],
    );

    fn on_open_level(
        &mut self,
        engine: &mut Engine,
        level: &mut Level,
        application: &mut Application,
        contents: &[EContentFileType],
    );

    fn tick(
        &mut self,
        engine: &mut Engine,
        ctx: egui::Context,
        contents: &[EContentFileType],
        application: &mut Application,
        #[cfg(not(target_os = "android"))] window: &mut winit::window::Window,
    );

    #[cfg(not(target_os = "android"))]
    fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent);

    fn on_window_input(
        &mut self,
        #[cfg(not(target_os = "android"))] window: &mut winit::window::Window,
        ctx: egui::Context,
        ty: crate::input_type::EInputType,
    ) -> Vec<winit::keyboard::KeyCode>;

    #[cfg(feature = "network")]
    fn as_network_replicated(&mut self) -> Option<&mut dyn crate::network::NetworkReplicated> {
        unimplemented!();
    }

    #[cfg(feature = "network")]
    fn as_network_module(&mut self) -> Option<&mut dyn crate::network::NetworkModule> {
        unimplemented!();
    }
}
