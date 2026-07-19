pub mod egui_render;

use crate::egui_render::EGUIRenderOutput;
use egui::{Pos2, ViewportId, ViewportInfo};
use egui_winit::State;
use winit::dpi::PhysicalSize;

pub fn ui_begin(egui_winit_state: &mut State, window: &mut winit::window::Window) {
    let ctx = egui_winit_state.egui_ctx().clone();
    let viewport_id = egui_winit_state.egui_input().viewport_id;
    let viewport_info: &mut egui::ViewportInfo = egui_winit_state
        .egui_input_mut()
        .viewports
        .get_mut(&viewport_id)
        .expect("Valid viewport id");
    egui_winit::update_viewport_info(viewport_info, &ctx, window, true);

    let new_input = egui_winit_state.take_egui_input(window);
    egui_winit_state.egui_ctx().begin_pass(new_input);
    egui_winit_state.egui_ctx().clear_animations();
}

pub fn ui_end(
    egui_winit_state: &mut State,
    window: &mut winit::window::Window,
) -> EGUIRenderOutput {
    let full_output = egui_winit_state.egui_ctx().end_pass();

    egui_winit_state.handle_platform_output(window, full_output.platform_output.clone());

    let viewport_id = egui_winit_state.egui_input().viewport_id;
    let viewport_info: &mut egui::ViewportInfo = egui_winit_state
        .egui_input_mut()
        .viewports
        .get_mut(&viewport_id)
        .expect("Valid viewport id");
    let native_pixels_per_point = viewport_info.native_pixels_per_point.unwrap_or(1.0);
    let pixels_per_point = pixels_per_point(egui_winit_state.egui_ctx(), native_pixels_per_point);
    let gui_render_output = EGUIRenderOutput {
        textures_delta: full_output.textures_delta,
        clipped_primitives: egui_winit_state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point),
        pixels_per_point,
    };
    gui_render_output
}

fn pixels_per_point(egui_ctx: &egui::Context, native_pixels_per_point: f32) -> f32 {
    let egui_zoom_factor = egui_ctx.zoom_factor();
    egui_zoom_factor * native_pixels_per_point
}

fn inner_rect_in_points(
    pixels_per_point: f32,
    inner_pos_px: Option<Pos2>,
    inner_size_px: egui::Vec2,
) -> Option<egui::Rect> {
    let inner_rect_px = egui::Rect::from_min_size(inner_pos_px?, inner_size_px);
    Some(inner_rect_px / pixels_per_point)
}

fn update_viewport_info(
    viewport_info: &mut ViewportInfo,
    egui_ctx: &egui::Context,
    is_init: bool,
    title: Option<String>,
    is_minimized: Option<bool>,
    is_maximized: Option<bool>,
    native_pixels_per_point: f32,
    inner_pos_px: Option<Pos2>,
    inner_size_px: egui::Vec2,
    monitor_size: Option<PhysicalSize<u32>>,
    fullscreen: Option<bool>,
    has_focus: Option<bool>,
) {
    let pixels_per_point = pixels_per_point(egui_ctx, native_pixels_per_point);

    let has_a_position = match is_minimized {
        Some(true) => false,
        Some(false) | None => true,
    };

    let inner_rect = if has_a_position {
        inner_rect_in_points(pixels_per_point, inner_pos_px, inner_size_px)
    } else {
        None
    };

    let outer_rect = if has_a_position {
        inner_rect_in_points(pixels_per_point, inner_pos_px, inner_size_px)
    } else {
        None
    };

    let monitor_size = {
        if let Some(monitor_size) = monitor_size {
            let size = monitor_size.to_logical::<f32>(pixels_per_point.into());
            Some(egui::vec2(size.width, size.height))
        } else {
            None
        }
    };

    viewport_info.title = title;
    viewport_info.native_pixels_per_point = Some(native_pixels_per_point);

    viewport_info.monitor_size = monitor_size;
    viewport_info.inner_rect = inner_rect;
    viewport_info.outer_rect = outer_rect;

    if is_init || !cfg!(target_os = "macos") {
        viewport_info.maximized = is_maximized;
        viewport_info.minimized = Some(is_minimized.unwrap_or(false));
    }

    viewport_info.fullscreen = fullscreen;
    viewport_info.focused = has_focus;
}

fn screen_size_in_pixels(
    outer_size: PhysicalSize<u32>,
    inner_size: PhysicalSize<u32>,
) -> egui::Vec2 {
    let size = if cfg!(target_os = "ios") {
        outer_size
    } else {
        inner_size
    };
    egui::vec2(size.width as f32, size.height as f32)
}

fn take_egui_input(
    egui_ctx: &egui::Context,
    egui_input: &mut egui::RawInput,
    elapsed: Option<f64>,
    outer_size: PhysicalSize<u32>,
    inner_size: PhysicalSize<u32>,
    native_pixels_per_point: f32,
    viewport_id: ViewportId,
) -> egui::RawInput {
    egui_input.time = Some(elapsed.unwrap_or(egui_ctx.time()));

    let screen_size_in_pixels = screen_size_in_pixels(outer_size, inner_size);
    let screen_size_in_points =
        screen_size_in_pixels / pixels_per_point(egui_ctx, native_pixels_per_point);

    egui_input.screen_rect = (screen_size_in_points.x > 0.0 && screen_size_in_points.y > 0.0)
        .then(|| egui::Rect::from_min_size(Pos2::ZERO, screen_size_in_points));

    egui_input.viewport_id = viewport_id;

    egui_input
        .viewports
        .entry(viewport_id)
        .or_default()
        .native_pixels_per_point = Some(native_pixels_per_point);

    egui_input.take()
}

fn _ui_begin_windowless(
    egui_ctx: &egui::Context,
    egui_input: &mut egui::RawInput,
    native_pixels_per_point: f32,
    inner_size: PhysicalSize<u32>,
    outer_size: PhysicalSize<u32>,
    title: Option<String>,
    inner_pos_px: Option<Pos2>,
    is_minimized: Option<bool>,
    is_maximized: Option<bool>,
    fullscreen: Option<bool>,
    has_focus: Option<bool>,
    elapsed: Option<f64>,
    monitor_size: Option<PhysicalSize<u32>>,
) {
    let viewport_id = egui_input.viewport_id;
    let viewport_info: &mut egui::ViewportInfo = egui_input
        .viewports
        .get_mut(&viewport_id)
        .expect("Valid viewport id");
    update_viewport_info(
        viewport_info,
        egui_ctx,
        true,
        title,
        is_minimized,
        is_maximized,
        native_pixels_per_point,
        inner_pos_px,
        egui::Vec2 {
            x: inner_size.width as f32,
            y: inner_size.height as f32,
        },
        monitor_size,
        fullscreen,
        has_focus,
    );

    let new_input = take_egui_input(
        egui_ctx,
        egui_input,
        elapsed,
        outer_size,
        inner_size,
        native_pixels_per_point,
        viewport_id,
    );
    egui_ctx.begin_pass(new_input);
    egui_ctx.clear_animations();
}

pub fn ui_begin_windowless(
    egui_ctx: &egui::Context,
    egui_input: &mut egui::RawInput,
    native_pixels_per_point: f32,
    inner_size: PhysicalSize<u32>,
) {
    _ui_begin_windowless(
        egui_ctx,
        egui_input,
        native_pixels_per_point,
        inner_size,
        inner_size,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
}

pub fn ui_end_windowless(
    egui_ctx: &egui::Context,
    egui_input: &mut egui::RawInput,
) -> EGUIRenderOutput {
    let viewport_id = egui_input.viewport_id;
    let viewport_info: &mut egui::ViewportInfo = egui_input
        .viewports
        .get_mut(&viewport_id)
        .expect("Valid viewport id");
    let native_pixels_per_point = viewport_info.native_pixels_per_point.unwrap_or(1.0);
    let pixels_per_point = pixels_per_point(egui_ctx, native_pixels_per_point);

    let full_output = egui_ctx.end_pass();

    let gui_render_output = EGUIRenderOutput {
        textures_delta: full_output.textures_delta,
        clipped_primitives: egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point),
        pixels_per_point,
    };
    gui_render_output
}
