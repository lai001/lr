use super::{
    misc::{self, update_window_with_input_mode},
    ui_window::UIWindow,
};
use crate::{editor_context::EWindowType, windows_manager::WindowsManager};
use anyhow::anyhow;
use egui_winit::State;
use rs_engine::{
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    input_mode::EInputMode,
};
use std::collections::HashMap;
use winit::event::WindowEvent;

pub struct ActorUIWindow {
    pub egui_winit_state: State,
    frame_sync: FrameSync,
    virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
}

impl ActorUIWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
    ) -> anyhow::Result<ActorUIWindow> {
        let window_context =
            window_manager.spwan_new_window(EWindowType::Actor, event_loop_window_target)?;
        let window = &*window_context.window.borrow();

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

        let input_mode = EInputMode::Game;
        update_window_with_input_mode(window, input_mode);

        Ok(ActorUIWindow {
            egui_winit_state,
            frame_sync,
            virtual_key_code_states: HashMap::new(),
        })
    }
}

impl UIWindow for ActorUIWindow {
    fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent) {
        let _ = device_event;
    }

    fn on_window_event(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
        window_manager: &mut WindowsManager,
        is_close: &mut bool,
    ) {
        let _ = event_loop_window_target;
        let _ = is_close;
        misc::on_window_event(
            window_id,
            EWindowType::Actor,
            window,
            &mut self.frame_sync,
            event,
            engine,
            window_manager,
            &mut self.virtual_key_code_states,
            60.0,
        );
    }
}
