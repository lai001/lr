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
