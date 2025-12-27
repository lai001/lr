// #[derive(Clone)]
pub enum EInputType<'a> {
    // Device(&'a winit::event::DeviceEvent),
    MouseWheel(&'a winit::event::MouseScrollDelta),
    MouseInput(
        &'a winit::event::ElementState,
        &'a winit::event::MouseButton,
    ),
    KeyboardInput(&'a mut crate::keys_detector::KeysDetector),
    CursorEntered,
    CursorLeft,
    CursorMoved(&'a winit::dpi::PhysicalPosition<f64>),
}
