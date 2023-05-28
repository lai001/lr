#[repr(C)]
#[derive(Debug)]
pub struct NativeKeyboardInput {
    pub scancode: winit::event::ScanCode,
    pub state: i32,
    pub virtual_key_code: i32,
}
