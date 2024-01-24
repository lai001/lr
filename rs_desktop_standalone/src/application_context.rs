use crate::custom_event::ECustomEventType;
use rs_artifact::{artifact::ArtifactReader, EEndianType};
use rs_engine::engine::Engine;
use std::path::Path;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

struct State {
    target_fps: u64,
    current_frame_start_time: std::time::Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            target_fps: 60,
            current_frame_start_time: std::time::Instant::now(),
        }
    }
}

pub struct ApplicationContext {
    engine: Engine,
    state: State,
    platform: egui_winit_platform::Platform,
}

impl ApplicationContext {
    pub fn new(window: &winit::window::Window) -> ApplicationContext {
        rs_foundation::change_working_directory();
        let window_size = window.inner_size();
        let scale_factor = 1.0f32;
        let window_width = window_size.width;
        let window_height = window_size.height;
        let descriptor = egui_winit_platform::PlatformDescriptor {
            physical_width: window_width,
            physical_height: window_height,
            scale_factor: scale_factor as f64,
            font_definitions: egui::FontDefinitions::default(),
            style: egui::Style::default(),
        };
        let platform = egui_winit_platform::Platform::new(descriptor);
        let artifact_filepath = Path::new("./main.rs");
        let artifact_reader =
            ArtifactReader::new(artifact_filepath, Some(EEndianType::Little)).unwrap();
        let engine = rs_engine::engine::Engine::new(
            window,
            window_width,
            window_height,
            scale_factor,
            platform.context(),
            Some(artifact_reader),
        )
        .unwrap();

        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .unwrap();
        window.set_cursor_visible(false);

        let application_context = Self {
            engine,
            state: State::default(),
            platform,
        };

        application_context
    }

    pub fn handle_event(
        &mut self,
        window: &mut winit::window::Window,
        event: &Event<ECustomEventType>,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            Event::DeviceEvent { device_id, event } => {
                self.engine.process_device_event(event.clone());
            }
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => {
                    self.engine.process_keyboard_input(*input);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                self.control_fps(control_flow);
                let full_output = self.process_ui(event_loop_proxy);
                self.engine.redraw(full_output);
            }
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    }

    fn process_ui(
        &mut self,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
    ) -> egui::FullOutput {
        self.platform.begin_frame();
        let full_output = self.platform.end_frame(None);
        full_output
    }

    fn control_fps(&mut self, control_flow: &mut ControlFlow) {
        let elapsed = std::time::Instant::now() - self.state.current_frame_start_time;
        Self::sync_fps(elapsed, self.state.target_fps, control_flow);
        self.state.current_frame_start_time = std::time::Instant::now();
    }

    fn sync_fps(
        elapsed: std::time::Duration,
        fps: u64,
        control_flow: &mut winit::event_loop::ControlFlow,
    ) {
        let fps = std::time::Duration::from_secs_f32(1.0 / fps as f32);
        let wait: std::time::Duration;
        if fps < elapsed {
            wait = std::time::Duration::from_millis(0);
        } else {
            wait = fps - elapsed;
        }
        let new_inst = std::time::Instant::now() + wait;
        *control_flow = winit::event_loop::ControlFlow::WaitUntil(new_inst);
    }
}
