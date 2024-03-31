use crate::custom_event::ECustomEventType;
use rs_artifact::{artifact::ArtifactReader, EEndianType};
use rs_engine::{
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    logger::{Logger, LoggerConfiguration},
};
use std::{collections::HashMap, path::Path};
use winit::event::{Event, WindowEvent};

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
    egui_winit_state: egui_winit::State,
    frame_sync: FrameSync,
}

impl ApplicationContext {
    pub fn new(window: &winit::window::Window) -> ApplicationContext {
        rs_foundation::change_working_directory();
        let logger = Logger::new(LoggerConfiguration {
            is_write_to_file: true,
        });
        let window_size = window.inner_size();
        let scale_factor = 1.0f32;
        let window_width = window_size.width;
        let window_height = window_size.height;

        let egui_context = egui::Context::default();
        egui_context.set_fonts(egui::FontDefinitions::default());
        egui_context.set_style(egui::Style::default());
        let egui_winit_state = egui_winit::State::new(
            egui_context,
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
        );
        let artifact_filepath = Path::new("./main.rs");
        let artifact_reader =
            ArtifactReader::new(artifact_filepath, Some(EEndianType::Little)).unwrap();
        let engine = rs_engine::engine::Engine::new(
            window,
            window_width,
            window_height,
            scale_factor,
            logger,
            Some(artifact_reader),
            HashMap::new(),
        )
        .unwrap();

        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .unwrap();
        window.set_cursor_visible(false);
        let frame_sync = FrameSync::new(EOptions::FPS(60.0));

        let application_context = Self {
            engine,
            state: State::default(),
            egui_winit_state,
            frame_sync,
        };

        application_context
    }

    pub fn handle_event(
        &mut self,
        window: &mut winit::window::Window,
        event: &Event<ECustomEventType>,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
    ) {
        match event {
            Event::DeviceEvent { device_id, event } => {
                self.engine.process_device_event(event.clone());
            }
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::CloseRequested => {
                    self.quit_app();
                }
                WindowEvent::KeyboardInput {
                    device_id,
                    event,
                    is_synthetic,
                } => {
                    self.engine
                        .process_keyboard_input(*device_id, event.clone(), *is_synthetic);
                }
                WindowEvent::RedrawRequested => {
                    let full_output = self.process_ui(window, event_loop_proxy);
                    let gui_render_output = rs_render::egui_render::EGUIRenderOutput {
                        textures_delta: full_output.textures_delta,
                        clipped_primitives: self
                            .egui_winit_state
                            .egui_ctx()
                            .tessellate(full_output.shapes, full_output.pixels_per_point),
                    };
                    self.engine.redraw(gui_render_output);
                    self.engine.present();
                    let wait = self
                        .frame_sync
                        .tick()
                        .unwrap_or(std::time::Duration::from_secs_f32(1.0 / 60.0));
                    std::thread::sleep(wait);
                    window.request_redraw();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn quit_app(&mut self) {
        std::process::exit(0);
    }

    fn process_ui(
        &mut self,
        window: &mut winit::window::Window,
        event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
    ) -> egui::FullOutput {
        let new_input = self.egui_winit_state.take_egui_input(window);
        self.egui_winit_state.egui_ctx().begin_frame(new_input);
        let full_output = self.egui_winit_state.egui_ctx().end_frame();
        full_output
    }
}
