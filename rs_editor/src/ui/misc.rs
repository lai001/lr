use crate::{editor_context::EWindowType, windows_manager::WindowsManager};
use egui_winit::State;
use rapier3d::prelude::RigidBodyType;
use rs_engine::{engine::Engine, frame_sync::FrameSync, input_mode::EInputMode};
use rs_render::egui_render::EGUIRenderOutput;
use std::collections::HashMap;
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::KeyCode,
    window::{CursorGrabMode, Window},
};

pub fn update_window_with_input_mode(window: &Window, input_mode: EInputMode) {
    match input_mode {
        EInputMode::Game => {
            window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
            window.set_cursor_visible(false);
        }
        EInputMode::UI => {
            window.set_cursor_grab(CursorGrabMode::None).unwrap();
            window.set_cursor_visible(true);
        }
        EInputMode::GameUI => {
            window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
            window.set_cursor_visible(true);
        }
    }
}

pub fn gui_render_output(
    window_id: isize,
    window: &Window,
    egui_winit_state: &mut State,
    add_contents: impl FnOnce(&mut State),
) -> EGUIRenderOutput {
    {
        let ctx = egui_winit_state.egui_ctx().clone();
        let viewport_id = egui_winit_state.egui_input().viewport_id;
        let viewport_info: &mut egui::ViewportInfo = egui_winit_state
            .egui_input_mut()
            .viewports
            .get_mut(&viewport_id)
            .unwrap();
        egui_winit::update_viewport_info(viewport_info, &ctx, window, true);
    }

    let new_input = egui_winit_state.take_egui_input(window);

    egui_winit_state.egui_ctx().begin_pass(new_input);

    add_contents(egui_winit_state);

    egui_winit_state.egui_ctx().clear_animations();

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

pub fn ui_begin(egui_winit_state: &mut State, window: &mut winit::window::Window) {
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

pub fn ui_end(
    egui_winit_state: &mut State,
    window: &mut winit::window::Window,
    window_id: isize,
) -> EGUIRenderOutput {
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

pub fn on_window_event(
    window_id: isize,
    window_type: EWindowType,
    window: &mut winit::window::Window,
    frame_sync: &mut FrameSync,
    event: &WindowEvent,
    engine: &mut Engine,
    window_manager: &mut WindowsManager,
    virtual_key_code_states: &mut HashMap<KeyCode, ElementState>,
    taget_fps: f32,
) {
    let _ = window_type;
    let _ = window_manager;
    let _ = window_id;
    match event {
        WindowEvent::KeyboardInput { event, .. } => {
            let winit::keyboard::PhysicalKey::Code(virtual_keycode) = event.physical_key else {
                return;
            };
            virtual_key_code_states.insert(virtual_keycode, event.state);
        }
        WindowEvent::RedrawRequested => {
            engine.tick();
            frame_sync.sync(taget_fps);
            window.request_redraw();
        }
        _ => {}
    }
}

pub fn render_combo_box2<'a, Value>(
    ui: &mut egui::Ui,
    label: &str,
    current_value: &mut Option<&'a Value>,
    selected_collection: Vec<Option<&'a Value>>,
) -> bool
where
    Value: ToUIString + std::cmp::PartialEq,
{
    let mut is_changed = false;
    let combo_box = egui::ComboBox::from_label(label).selected_text(format!("{}", {
        match current_value {
            Some(current_url) => current_url.to_ui_string(),
            None => "None".to_string(),
        }
    }));
    combo_box.show_ui(ui, |ui| {
        for selected_value in selected_collection {
            let text = selected_value
                .as_ref()
                .map(|x| x.to_ui_string())
                .unwrap_or("None".to_string());
            is_changed = ui
                .selectable_value(current_value, selected_value, text)
                .changed();
            if is_changed {
                break;
            }
        }
    });
    is_changed
}

pub fn render_combo_box<'a, Value>(
    ui: &mut egui::Ui,
    label: &str,
    current_value: &mut Option<&'a Value>,
    candidate_items: &'a Vec<Value>,
) -> bool
where
    Value: ToUIString + std::cmp::PartialEq,
{
    let mut selected_collection: Vec<Option<&Value>> =
        Vec::with_capacity(1 + candidate_items.len());
    selected_collection.push(None);
    selected_collection.append(&mut candidate_items.iter().map(|x| Some(x)).collect());
    render_combo_box2(ui, label, current_value, selected_collection)
}

pub fn render_combo_box_not_null<Value>(
    ui: &mut egui::Ui,
    label: &str,
    current_value: &mut Value,
    selected_collection: Vec<Value>,
) -> bool
where
    Value: ToUIString + std::cmp::PartialEq,
{
    let mut is_changed = false;
    let combo_box = egui::ComboBox::from_label(label)
        .selected_text(format!("{}", { current_value.to_ui_string() }));
    combo_box.show_ui(ui, |ui| {
        for selected_value in selected_collection {
            let text = selected_value.to_ui_string();
            is_changed = ui
                .selectable_value(current_value, selected_value, text)
                .changed();
            if is_changed {
                break;
            }
        }
    });
    is_changed
}

pub trait ToUIString {
    fn to_ui_string(&self) -> String;
}

impl ToUIString for RigidBodyType {
    fn to_ui_string(&self) -> String {
        match self {
            RigidBodyType::Dynamic => "Dynamic".to_string(),
            RigidBodyType::Fixed => "Fixed".to_string(),
            RigidBodyType::KinematicPositionBased => "Kinematic position based".to_string(),
            RigidBodyType::KinematicVelocityBased => "Kinematic velocity based".to_string(),
        }
    }
}

impl ToUIString for url::Url {
    fn to_ui_string(&self) -> String {
        self.to_string()
    }
}
