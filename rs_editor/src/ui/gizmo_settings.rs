use egui::{color_picker::Alpha, Context, Widget};
use transform_gizmo_egui::*;

fn gizmo_mode_text(gizmo_mode: &EnumSet<GizmoMode>) -> &'static str {
    if gizmo_mode == &GizmoMode::all_rotate() {
        "Rotate"
    } else if gizmo_mode == &GizmoMode::all_scale() {
        "Scale"
    } else if gizmo_mode == &GizmoMode::all_translate() {
        "Translate"
    } else if gizmo_mode == &GizmoMode::all() {
        "All"
    } else {
        unreachable!()
    }
}

pub fn draw(
    window: egui::Window,
    context: &Context,
    visuals: &mut GizmoVisuals,
    gizmo_mode: &mut EnumSet<GizmoMode>,
    gizmo_orientation: &mut GizmoOrientation,
    custom_highlight_color: &mut bool,
    is_open: &mut bool,
) {
    let stroke_width = &mut visuals.stroke_width;
    let gizmo_size = &mut visuals.gizmo_size;
    let mut highlight_color = egui::Color32::GOLD;
    let x_color = &mut visuals.x_color;
    let y_color = &mut visuals.y_color;
    let z_color = &mut visuals.z_color;
    let s_color = &mut visuals.s_color;
    let inactive_alpha = &mut visuals.inactive_alpha;
    let highlight_alpha = &mut visuals.highlight_alpha;

    window
        .resizable(false)
        .open(is_open)
        .default_open(false)
        .show(context, |ui| {
            egui::ComboBox::from_label("Mode")
                .selected_text(gizmo_mode_text(gizmo_mode))
                .show_ui(ui, |ui| {
                    for mode in [
                        GizmoMode::all_rotate(),
                        GizmoMode::all_scale(),
                        GizmoMode::all_translate(),
                        GizmoMode::all(),
                    ] {
                        ui.selectable_value(gizmo_mode, mode, gizmo_mode_text(&mode));
                    }
                });
            ui.end_row();

            egui::ComboBox::from_label("Orientation")
                .selected_text(format!("{gizmo_orientation:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(gizmo_orientation, GizmoOrientation::Global, "Global");
                    ui.selectable_value(gizmo_orientation, GizmoOrientation::Local, "Local");
                });
            ui.end_row();

            ui.separator();

            egui::Slider::new(gizmo_size, 10.0f32..=500.0)
                .text("Gizmo size")
                .ui(ui);
            egui::Slider::new(stroke_width, 0.1..=10.0)
                .text("Stroke width")
                .ui(ui);
            egui::Slider::new(inactive_alpha, 0.0..=1.0)
                .text("Inactive alpha")
                .ui(ui);
            egui::Slider::new(highlight_alpha, 0.0..=1.0)
                .text("Highlighted alpha")
                .ui(ui);

            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut highlight_color,
                    Alpha::Opaque,
                );
                egui::Checkbox::new(custom_highlight_color, "Custom highlight color").ui(ui);
            });

            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(ui, x_color, Alpha::Opaque);
                egui::Label::new("X axis color").ui(ui);
            });

            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(ui, y_color, Alpha::Opaque);
                egui::Label::new("Y axis color").ui(ui);
            });
            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(ui, z_color, Alpha::Opaque);
                egui::Label::new("Z axis color").ui(ui);
            });
            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(ui, s_color, Alpha::Opaque);
                egui::Label::new("Screen axis color").ui(ui);
            });
            ui.end_row();
        });
    if *custom_highlight_color {
        visuals.highlight_color = Some(highlight_color);
    } else {
        visuals.highlight_color = None;
    }
}
