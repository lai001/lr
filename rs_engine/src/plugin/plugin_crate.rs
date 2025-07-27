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
        #[cfg(not(target_os = "android"))] window: &mut winit::window::Window,
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

#[cfg(feature = "network")]
impl crate::network::NetworkReplicated for dyn Plugin {
    fn get_network_id(&self) -> &uuid::Uuid {
        unimplemented!();
    }

    fn set_network_id(&mut self, _: uuid::Uuid) {
        unimplemented!();
    }

    fn is_replicated(&self) -> bool {
        unimplemented!();
    }

    fn set_replicated(&mut self, _: bool) {
        unimplemented!();
    }

    fn sync_with_server(&mut self, _: bool) {
        unimplemented!();
    }

    fn is_sync_with_server(&self) -> bool {
        unimplemented!();
    }
}
