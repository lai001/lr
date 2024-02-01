use crate::{custom_event::ECustomEventType, editor_context::EditorContext};
use winit::event_loop::EventLoopBuilder;

pub struct Editor {
    event_loop: winit::event_loop::EventLoop<ECustomEventType>,
    event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
    window: winit::window::Window,
    editor_context: EditorContext,
}

impl Editor {
    pub fn new() -> Self {
        let window_width = 1280;
        let window_height = 720;
        let event_loop = EventLoopBuilder::with_user_event().build();
        let event_loop_proxy = event_loop.create_proxy();
        let window = winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title("Editor")
            .with_inner_size(winit::dpi::PhysicalSize {
                width: window_width,
                height: window_height,
            })
            .build(&event_loop)
            .unwrap();
        window.set_ime_allowed(true);
        let editor_context = EditorContext::new(&window);

        Self {
            editor_context,
            event_loop,
            event_loop_proxy,
            window,
        }
    }

    pub fn run(mut self) {
        self.event_loop.run({
            move |event, _, control_flow| {
                self.editor_context.handle_event(
                    &mut self.window,
                    &event,
                    self.event_loop_proxy.clone(),
                    control_flow,
                );
            }
        });
    }
}