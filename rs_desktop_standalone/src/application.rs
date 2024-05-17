use crate::{application_context::ApplicationContext, custom_event::ECustomEventType};
use clap::Parser;
use winit::event_loop::EventLoopBuilder;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: Option<std::path::PathBuf>,
}

pub struct Application {
    event_loop: winit::event_loop::EventLoop<ECustomEventType>,
    event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
    window: winit::window::Window,
    application_context: ApplicationContext,
}

impl Application {
    pub fn new() -> Self {
        let args = Args::parse();
        let window_width = 1280;
        let window_height = 720;
        let event_loop = EventLoopBuilder::with_user_event().build().unwrap();
        let event_loop_proxy = event_loop.create_proxy();
        let window = winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title("Standalone")
            .with_inner_size(winit::dpi::PhysicalSize {
                width: window_width,
                height: window_height,
            })
            .build(&event_loop)
            .unwrap();
        window.set_ime_allowed(true);
        let application_context = ApplicationContext::new(&window, args.input_file);

        Self {
            application_context,
            event_loop,
            event_loop_proxy,
            window,
        }
    }

    pub fn run(mut self) {
        let result = self.event_loop.run({
            move |event, event_loop_window_target| {
                self.application_context.handle_event(
                    &mut self.window,
                    &event,
                    self.event_loop_proxy.clone(),
                    event_loop_window_target,
                );
            }
        });
        log::trace!("{:?}", result);
    }
}
