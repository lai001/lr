use super::misc::update_window_with_input_mode;
use crate::{custom_event::ECustomEventType, editor::WindowsManager, editor_context::EWindowType};
use anyhow::anyhow;
use egui_winit::State;
use rs_engine::{
    content::{content_file_type::EContentFileType, level::Level},
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    input_mode::EInputMode,
};
use rs_render::{command::RenderCommand, egui_render::EGUIRenderOutput};
use rs_standalone_core::application::Application;
use std::collections::HashMap;
use winit::event::WindowEvent;

pub struct StandaloneUiWindow {
    pub egui_winit_state: State,
    frame_sync: FrameSync,
    virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
    input_mode: EInputMode,
    application: Application,
}

impl StandaloneUiWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
        engine: &mut Engine,
        plugins: Vec<Box<dyn rs_native_plugin::Plugin>>,
        active_level: &Level,
        contents: Vec<EContentFileType>,
    ) -> anyhow::Result<StandaloneUiWindow> {
        let window_context =
            window_manager.spwan_new_window(EWindowType::Standalone, event_loop_window_target)?;
        let window = &*window_context.window.borrow();
        let window_id = window_context.get_id();

        engine
            .set_new_window(
                window_context.get_id(),
                window,
                window_context.get_width(),
                window_context.get_height(),
                window.scale_factor() as f32,
            )
            .map_err(|err| anyhow!("{err}"))?;
        let viewport_id = egui::ViewportId::from_hash_of(window_context.get_id());

        let mut egui_winit_state = egui_winit::State::new(
            context,
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
        );

        egui_winit_state.egui_input_mut().viewport_id = viewport_id;
        egui_winit_state.egui_input_mut().viewports =
            std::iter::once((viewport_id, Default::default())).collect();

        let frame_sync = FrameSync::new(EOptions::FPS(60.0));

        let input_mode = EInputMode::Game;
        update_window_with_input_mode(window, input_mode);

        let level = active_level.make_copy_for_standalone(engine);

        let application = Application::new(
            window_id,
            window.inner_size().width,
            window.inner_size().height,
            engine,
            level,
            plugins,
            contents,
            input_mode,
        );
        Ok(StandaloneUiWindow {
            egui_winit_state,
            frame_sync,
            virtual_key_code_states: HashMap::new(),
            input_mode,
            application,
        })
    }

    pub fn device_event_process(&mut self, device_event: &winit::event::DeviceEvent) {
        self.application
            .on_input(rs_native_plugin::EInputType::Device(device_event));
    }

    pub fn window_event_process(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
        engine: &mut Engine,
        window_manager: &mut WindowsManager,
    ) {
        let _ = event_loop_window_target;
        let _ = self.egui_winit_state.on_window_event(window, event);
        update_window_with_input_mode(window, self.input_mode);

        match event {
            WindowEvent::Resized(size) => {
                engine.resize(window_id, size.width, size.height);
                self.application.on_size_changed(size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                window_manager.remove_window(EWindowType::Standalone);
                engine.remove_window(window_id);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let winit::keyboard::PhysicalKey::Code(virtual_keycode) = event.physical_key else {
                    return;
                };
                self.virtual_key_code_states
                    .insert(virtual_keycode, event.state);
                self.application
                    .on_input(rs_native_plugin::EInputType::KeyboardInput(
                        &self.virtual_key_code_states,
                    ));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.application
                    .on_input(rs_native_plugin::EInputType::MouseWheel(delta));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.application
                    .on_input(rs_native_plugin::EInputType::MouseInput(state, button));
            }
            WindowEvent::RedrawRequested => {
                engine.recv_output_hook();

                self.ui_begin(window);

                self.application.on_redraw_requested(
                    engine,
                    self.egui_winit_state.egui_ctx().clone(),
                    &self.virtual_key_code_states,
                );

                engine.send_render_command(RenderCommand::UiOutput(self.ui_end(window, window_id)));
                self.sync(window);
            }
            _ => {}
        }
    }

    fn sync(&mut self, window: &mut winit::window::Window) {
        let wait = self
            .frame_sync
            .tick()
            .unwrap_or(std::time::Duration::from_secs_f32(1.0 / 60.0));
        std::thread::sleep(wait);
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
        egui_winit_state.egui_ctx().begin_frame(new_input);
        egui_winit_state.egui_ctx().clear_animations();
    }

    fn ui_end(&mut self, window: &mut winit::window::Window, window_id: isize) -> EGUIRenderOutput {
        let egui_winit_state = &mut self.egui_winit_state;

        let full_output = egui_winit_state.egui_ctx().end_frame();

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

    pub fn reload_plugins(&mut self, plugins: Vec<Box<dyn rs_native_plugin::Plugin>>) {
        self.application.reload_plugins(plugins);
    }
}
