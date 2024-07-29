use crate::custom_event::ECustomEventType;
use rs_artifact::{artifact::ArtifactReader, EEndianType};
use rs_engine::{
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    logger::{Logger, LoggerConfiguration},
};
use std::{collections::HashMap, path::Path};
use winit::event::{Event, MouseScrollDelta, WindowEvent};

pub struct ApplicationContext {
    engine: Engine,
    egui_winit_state: egui_winit::State,
    frame_sync: FrameSync,
}

impl ApplicationContext {
    pub fn new(
        window: &winit::window::Window,
        input_file: Option<impl AsRef<Path>>,
    ) -> ApplicationContext {
        let window_id = u64::from(window.id()) as isize;
        rs_foundation::change_working_directory();
        let logger = Logger::new(LoggerConfiguration {
            is_write_to_file: true,
            is_flush_before_drop: false,
        });
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
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
        let artifact_filepath = match input_file {
            Some(input_file) => input_file.as_ref().to_path_buf(),
            None => Path::new("main.rs").to_path_buf(),
        };
        let artifact_reader =
            ArtifactReader::new(&artifact_filepath, Some(EEndianType::Little)).ok();
        let mut engine = rs_engine::engine::Engine::new(
            window_id,
            window,
            window_width,
            window_height,
            scale_factor,
            logger,
            artifact_reader,
            HashMap::new(),
        )
        .unwrap();

        engine.init_level();

        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .unwrap();
        window.set_cursor_visible(false);
        let frame_sync = FrameSync::new(EOptions::FPS(60.0));

        let application_context = Self {
            engine,
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
        _: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
    ) {
        match event {
            Event::DeviceEvent { event, .. } => {
                self.engine.process_device_event(event.clone());
            }
            Event::WindowEvent { event, .. } => match event {
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
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(_, up) => {
                        let mut speed = self.engine.get_camera_movement_speed();
                        speed += up * 0.005;
                        speed = speed.max(0.0);
                        self.engine.set_camera_movement_speed(speed);
                    }
                    MouseScrollDelta::PixelDelta(_) => todo!(),
                },
                WindowEvent::RedrawRequested => {
                    let window_id = u64::from(window.id()) as isize;
                    let full_output = self.process_ui(window, event_loop_proxy);
                    let gui_render_output = rs_render::egui_render::EGUIRenderOutput {
                        textures_delta: full_output.textures_delta,
                        clipped_primitives: self
                            .egui_winit_state
                            .egui_ctx()
                            .tessellate(full_output.shapes, full_output.pixels_per_point),
                        window_id,
                    };
                    self.engine.tick();
                    self.engine.redraw(gui_render_output);
                    self.engine.present(window_id);
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
        _: winit::event_loop::EventLoopProxy<ECustomEventType>,
    ) -> egui::FullOutput {
        let new_input = self.egui_winit_state.take_egui_input(window);
        self.egui_winit_state.egui_ctx().begin_frame(new_input);
        let full_output = self.egui_winit_state.egui_ctx().end_frame();
        full_output
    }
}
