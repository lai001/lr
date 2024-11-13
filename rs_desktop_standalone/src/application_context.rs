use crate::custom_event::ECustomEventType;
use rs_artifact::{artifact::ArtifactReader, EEndianType};
use rs_engine::{
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    input_mode::EInputMode,
    input_type::EInputType,
    logger::{Logger, LoggerConfiguration, SlotFlags},
};
use rs_render::{command::RenderCommand, egui_render::EGUIRenderOutput};
use std::{collections::HashMap, path::Path};
use winit::event::{Event, WindowEvent};

include!("../target/generated/load_plugins.generated.rs");

pub struct ApplicationContext {
    engine: Engine,
    egui_winit_state: egui_winit::State,
    frame_sync: FrameSync,
    app: rs_engine::standalone::application::Application,
    virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
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
            slot_flags: SlotFlags::empty(),
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

        engine.init_resources();

        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .unwrap();
        window.set_cursor_visible(false);
        let frame_sync = FrameSync::new(EOptions::FPS(60.0));

        let current_active_level = engine.new_main_level().unwrap();
        let contents = engine
            .content_files
            .iter()
            .map(|(_, x)| x.clone())
            .collect();

        #[cfg(feature = "plugin_shared_crate")]
        let plugins = LoadPlugins::load();
        let app = rs_engine::standalone::application::Application::new(
            window_id,
            window_width,
            window_height,
            &mut engine,
            &current_active_level,
            contents,
            EInputMode::Game,
            #[cfg(feature = "plugin_shared_crate")]
            plugins,
        );

        let application_context = Self {
            engine,
            egui_winit_state,
            frame_sync,
            app,
            virtual_key_code_states: HashMap::new(),
        };

        application_context
    }

    pub fn handle_event(
        &mut self,
        window: &mut winit::window::Window,
        event: &Event<ECustomEventType>,
        // event_loop_proxy: winit::event_loop::EventLoopProxy<ECustomEventType>,
        // _: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
    ) {
        // let _ = event_loop_proxy;
        match event {
            Event::DeviceEvent { event, .. } => {
                self.app.on_device_event(event);
            }
            Event::WindowEvent { event, .. } => {
                let _ = self.egui_winit_state.on_window_event(window, event);
                // Engine::update_window_with_input_mode(window, EInputMode::Game);
                match event {
                    WindowEvent::CloseRequested => {
                        self.quit_app();
                    }
                    WindowEvent::CursorEntered { .. } => {
                        self.app.on_window_input(window, EInputType::CursorEntered);
                    }
                    WindowEvent::CursorLeft { .. } => {
                        self.app.on_window_input(window, EInputType::CursorLeft);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.app
                            .on_window_input(window, EInputType::CursorMoved(position));
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        let winit::keyboard::PhysicalKey::Code(virtual_keycode) =
                            event.physical_key
                        else {
                            return;
                        };
                        self.virtual_key_code_states
                            .insert(virtual_keycode, event.state);
                        let consume = self.app.on_window_input(
                            window,
                            EInputType::KeyboardInput(&self.virtual_key_code_states),
                        );
                        for item in consume {
                            let _ = self.virtual_key_code_states.remove(&item);
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        self.app
                            .on_window_input(window, EInputType::MouseWheel(delta));
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        self.app
                            .on_window_input(window, EInputType::MouseInput(state, button));
                    }
                    WindowEvent::RedrawRequested => {
                        let window_id = u64::from(window.id()) as isize;
                        self.engine.window_redraw_requested_begin(window_id);
                        self.ui_begin(window);

                        self.engine.tick();

                        self.app.on_redraw_requested(
                            &mut self.engine,
                            self.egui_winit_state.egui_ctx().clone(),
                            window,
                            &self.virtual_key_code_states,
                        );

                        let output = self.ui_end(window, window_id);
                        self.engine
                            .send_render_command(RenderCommand::UiOutput(output));
                        self.sync(window);
                        self.engine.window_redraw_requested_end(window_id);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn quit_app(&mut self) {
        std::process::exit(0);
    }

    fn sync(&mut self, window: &mut winit::window::Window) {
        self.frame_sync.sync(60.0);
        window.request_redraw();
    }

    fn ui_begin(&mut self, window: &mut winit::window::Window) {
        let egui_winit_state = &mut self.egui_winit_state;

        let ctx = egui_winit_state.egui_ctx().clone();
        let viewport_id = egui_winit_state.egui_input().viewport_id;
        let viewport_info: &mut egui::ViewportInfo = egui_winit_state
            .egui_input_mut()
            .viewports
            .get_mut(&viewport_id)
            .unwrap();
        egui_winit::update_viewport_info(viewport_info, &ctx, window, true);

        let new_input = egui_winit_state.take_egui_input(window);
        egui_winit_state.egui_ctx().begin_pass(new_input);
        egui_winit_state.egui_ctx().clear_animations();
    }

    fn ui_end(&mut self, window: &mut winit::window::Window, window_id: isize) -> EGUIRenderOutput {
        let egui_winit_state = &mut self.egui_winit_state;

        let full_output = egui_winit_state.egui_ctx().end_pass();

        egui_winit_state.handle_platform_output(window, full_output.platform_output.clone());

        let gui_render_output = rs_render::egui_render::EGUIRenderOutput {
            textures_delta: full_output.textures_delta,
            clipped_primitives: egui_winit_state
                .egui_ctx()
                .tessellate(full_output.shapes, full_output.pixels_per_point),
            window_id,
        };
        gui_render_output
    }
}
