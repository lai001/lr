const INITIAL_WIDTH: u32 = 1280;
const INITIAL_HEIGHT: u32 = 720;

pub struct NativeWindow {
    pub event_loop: winit::event_loop::EventLoop<()>,
    pub window: winit::window::Window,
}

impl NativeWindow {
    pub fn new() -> NativeWindow {
        let event_loop = winit::event_loop::EventLoop::default();
        let window = winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title("Example")
            .with_inner_size(winit::dpi::PhysicalSize {
                width: INITIAL_WIDTH,
                height: INITIAL_HEIGHT,
            })
            .build(&event_loop)
            .unwrap();

        NativeWindow { event_loop, window }
    }

    pub fn get_window_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }
}
