use super::{misc::update_window_with_input_mode, ui_window::UIWindow};
use crate::{
    editor_context::EWindowType, standalone_simulation_options::StandaloneSimulationType,
    windows_manager::WindowsManager,
};
use anyhow::anyhow;
use egui_winit::State;
#[cfg(feature = "plugin_shared_crate")]
use rs_engine::plugin::plugin_crate::Plugin;
use rs_engine::{
    content::{content_file_type::EContentFileType, level::Level},
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    input_mode::EInputMode,
    input_type::EInputType,
    standalone::application::Application,
};
use rs_render::command::{RenderCommand, ScaleChangedInfo};
use std::collections::HashMap;
use winit::{event::WindowEvent, keyboard::KeyCode};

pub struct StandaloneUiWindow {
    application: Application,
    pub egui_winit_state: State,
    frame_sync: FrameSync,
    virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
    input_mode: EInputMode,
    window_id: isize,
}

impl UIWindow for StandaloneUiWindow {
    fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent) {
        self.application.on_device_event(device_event);
    }

    fn on_window_event(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
        window_manager: &mut WindowsManager,
        is_request_close: &mut bool,
    ) {
        let _ = event_loop_window_target;
        let _ = self.egui_winit_state.on_window_event(window, event);
        update_window_with_input_mode(window, self.input_mode);
        super::misc::on_window_event(
            window_id,
            EWindowType::Standalone,
            window,
            &mut self.frame_sync,
            event,
            engine,
            window_manager,
            &mut self.virtual_key_code_states,
            60.0,
        );

        match event {
            WindowEvent::CursorEntered { .. } => {
                self.application
                    .on_window_input(window, EInputType::CursorEntered);
            }
            WindowEvent::CursorLeft { .. } => {
                self.application
                    .on_window_input(window, EInputType::CursorLeft);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.application
                    .on_window_input(window, EInputType::CursorMoved(position));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let winit::keyboard::PhysicalKey::Code(virtual_keycode) = event.physical_key else {
                    return;
                };
                if virtual_keycode == KeyCode::Escape {
                    *is_request_close = true;
                    return;
                }
                let consume = self.application.on_window_input(
                    window,
                    EInputType::KeyboardInput(&self.virtual_key_code_states),
                );
                for item in consume {
                    let _ = self.virtual_key_code_states.remove(&item);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.application
                    .on_window_input(window, EInputType::MouseWheel(delta));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.application
                    .on_window_input(window, EInputType::MouseInput(state, button));
            }
            WindowEvent::RedrawRequested => {
                engine.window_redraw_requested_begin(window_id);
                super::misc::ui_begin(&mut self.egui_winit_state, window);
                self.application.on_redraw_requested(
                    engine,
                    self.egui_winit_state.egui_ctx().clone(),
                    window,
                    &self.virtual_key_code_states,
                );
                engine.send_render_command(RenderCommand::UiOutput(super::misc::ui_end(
                    &mut self.egui_winit_state,
                    window,
                    window_id,
                )));

                engine.window_redraw_requested_end(window_id);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                engine.send_render_command(RenderCommand::ScaleChanged(ScaleChangedInfo {
                    window_id,
                    new_factor: *scale_factor as f32,
                }));
            }
            _ => {}
        }
    }

    fn get_window_id(&self) -> isize {
        self.window_id
    }
}

impl StandaloneUiWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
        #[cfg(feature = "plugin_shared_crate")] plugins: Vec<Box<dyn Plugin>>,
        active_level: &Level,
        contents: Vec<EContentFileType>,
        standalone_simulation_type: StandaloneSimulationType,
    ) -> anyhow::Result<StandaloneUiWindow> {
        let window_context = window_manager.spwan_new_window(
            EWindowType::Standalone,
            event_loop_window_target,
            Self::title_of(&standalone_simulation_type),
        )?;
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
            None,
        );

        egui_winit_state.egui_input_mut().viewport_id = viewport_id;
        egui_winit_state.egui_input_mut().viewports =
            std::iter::once((viewport_id, Default::default())).collect();

        let frame_sync = FrameSync::new(EOptions::FPS(60.0));

        // let input_mode = EInputMode::GameUI;
        let input_mode = EInputMode::UI;
        update_window_with_input_mode(window, input_mode);

        // let level = active_level.make_copy_for_standalone(engine, &contents);

        #[allow(unused_mut)]
        let mut application = Application::new(
            window_id,
            window.inner_size().width,
            window.inner_size().height,
            engine,
            active_level,
            contents,
            input_mode,
            #[cfg(feature = "plugin_shared_crate")]
            plugins,
        );
        #[cfg(feature = "network")]
        {
            let (server, client) = match &standalone_simulation_type {
                StandaloneSimulationType::Single => {
                    application.is_authority = true;
                    (None, None)
                }
                StandaloneSimulationType::MultiplePlayer(multiple_player_options) => {
                    if multiple_player_options.is_server {
                        application.is_authority = true;
                        let server = rs_network::server::Server::bind(
                            multiple_player_options.server_socket_addr,
                        );
                        if let Err(err) = &server {
                            log::warn!("{}", err);
                        }
                        (server.ok(), None)
                    } else {
                        application.is_authority = false;
                        let client = rs_network::client::Client::bind(
                            multiple_player_options.server_socket_addr,
                            Some("Client".to_string()),
                        );
                        if let Err(err) = &client {
                            log::warn!("{}", err);
                        }
                        (None, client.ok())
                    }
                }
            };
            application.server = server;
            application.client = client;
        }

        Ok(StandaloneUiWindow {
            egui_winit_state,
            frame_sync,
            virtual_key_code_states: HashMap::new(),
            input_mode,
            application,
            window_id,
        })
    }

    #[cfg(feature = "plugin_shared_crate")]
    pub fn reload_plugins(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        self.application.reload_plugins(plugins);
    }

    fn title_of(standalone_simulation_type: &StandaloneSimulationType) -> Option<String> {
        match standalone_simulation_type {
            StandaloneSimulationType::Single => None,
            StandaloneSimulationType::MultiplePlayer(multiple_player_options) => {
                if multiple_player_options.is_server {
                    Some(format!("Standalone({})", "Server"))
                } else {
                    Some(format!("Standalone({})", "Client"))
                }
            }
        }
    }
}
