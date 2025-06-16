use crate::windows_manager::WindowsManager;
use rs_engine::engine::Engine;
use winit::event::WindowEvent;

pub trait UIWindow {
    fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent);

    fn on_window_event(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
        window_manager: &mut WindowsManager,
        is_request_close: &mut bool,
    );

    fn get_window_id(&self) -> isize;
}
