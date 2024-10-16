use crate::{application_context::ApplicationContext, custom_event::ECustomEventType};
use clap::Parser;
use winit::{
    application::ApplicationHandler, event_loop::EventLoop,
    platform::windows::EventLoopBuilderExtWindows,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: Option<std::path::PathBuf>,
}

pub struct Application {
    window: Option<winit::window::Window>,
    application_context: Option<ApplicationContext>,
}

impl Application {
    pub fn new() -> Application {
        Application {
            application_context: None,
            window: None,
        }
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::<ECustomEventType>::with_user_event()
            .with_any_thread(true)
            .build()?;
        let result = event_loop.run_app(&mut self);
        Ok(result?)
    }
}

impl ApplicationHandler<ECustomEventType> for Application {
    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        match &cause {
            winit::event::StartCause::Init => {
                let args = Args::parse();
                let scale_factor = event_loop
                    .primary_monitor()
                    .map(|x| x.scale_factor())
                    .unwrap_or(1.0);
                let window_width = (1280 as f64 * scale_factor) as u32;
                let window_height = (720 as f64 * scale_factor) as u32;
                let window_attributes = winit::window::Window::default_attributes()
                    .with_decorations(true)
                    .with_resizable(true)
                    .with_transparent(false)
                    .with_title("Standalone")
                    .with_inner_size(winit::dpi::PhysicalSize {
                        width: window_width,
                        height: window_height,
                    });
                let window = event_loop
                    .create_window(window_attributes)
                    .expect("Should not be null");
                window.set_ime_allowed(true);
                let application_context = ApplicationContext::new(&window, args.input_file);
                self.window = Some(window);
                self.application_context = Some(application_context);
            }
            _ => {}
        }
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: ECustomEventType,
    ) {
        let _ = event_loop;
        let event = winit::event::Event::UserEvent(event);
        let window = self.window.as_mut().expect("Should not be null");
        self.application_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(window, &event);
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = event_loop;
        let event = winit::event::Event::DeviceEvent { device_id, event };
        let window = self.window.as_mut().expect("Should not be null");
        self.application_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(window, &event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
        let event = winit::event::Event::AboutToWait;
        let window = self.window.as_mut().expect("Should not be null");
        self.application_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(window, &event);
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
        let event = winit::event::Event::Suspended;
        let window = self.window.as_mut().expect("Should not be null");
        self.application_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(window, &event);
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
        let event = winit::event::Event::LoopExiting;
        let window = self.window.as_mut().expect("Should not be null");
        self.application_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(window, &event);
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
        let event = winit::event::Event::MemoryWarning;
        let window = self.window.as_mut().expect("Should not be null");
        self.application_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(window, &event);
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
        let event = winit::event::Event::Resumed;
        let window = self.window.as_mut().expect("Should not be null");
        self.application_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(window, &event);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let _ = event_loop;
        let event = winit::event::Event::WindowEvent { window_id, event };
        let window = self.window.as_mut().expect("Should not be null");
        self.application_context
            .as_mut()
            .expect("Should not be null")
            .handle_event(window, &event);
    }
}
