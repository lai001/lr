use crate::{custom_event::ECustomEventType, editor_context::EditorContext};
use winit::event_loop::EventLoopBuilder;

pub struct Editor {
    event_loop: winit::event_loop::EventLoop<ECustomEventType>,
    window: winit::window::Window,
    editor_context: EditorContext,
}

impl Editor {
    pub fn new() -> Self {
        let window_width = 1280;
        let window_height = 720;
        let event_loop = EventLoopBuilder::with_user_event().build().unwrap();
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
        let editor_context = EditorContext::new(&window, event_loop_proxy.clone());

        Self {
            editor_context,
            event_loop,
            window,
        }
    }

    pub fn run(mut self) {
        let event_loop_result = self.event_loop.run({
            move |event, event_loop_window_target| {
                self.editor_context.handle_event(
                    &mut self.window,
                    &event,
                    event_loop_window_target,
                );
            }
        });
        match event_loop_result {
            Ok(_) => {}
            Err(err) => {
                log::warn!("{}", err);
            }
        }
    }
}
