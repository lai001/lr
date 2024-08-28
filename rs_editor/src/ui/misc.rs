use egui_winit::State;
use rs_engine::input_mode::EInputMode;
use rs_render::egui_render::EGUIRenderOutput;
use winit::window::{CursorGrabMode, Window};

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

    egui_winit_state.egui_ctx().begin_frame(new_input);

    add_contents(egui_winit_state);

    egui_winit_state.egui_ctx().clear_animations();

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

pub fn random_color3() -> glam::Vec3 {
    let x: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
    let y: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
    let z: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
    glam::vec3(x, y, z)
}

pub fn random_color4() -> glam::Vec4 {
    let x: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
    let y: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
    let z: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
    let w: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
    glam::vec4(x, y, z, w)
}
