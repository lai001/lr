use crate::data_source::{AssetFile, AssetFolder};
use egui::{color_picker::Alpha, Color32, Context, RichText, Ui, Widget, Window};
use egui_gizmo::{GizmoMode, GizmoOrientation, GizmoVisuals};
use rs_engine::file_type::EFileType;

pub fn draw(
    context: &Context,
    visuals: &mut GizmoVisuals,
    gizmo_mode: &mut GizmoMode,
    gizmo_orientation: &mut GizmoOrientation,
    custom_highlight_color: &mut bool,
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

    egui::Window::new("Gizmo Settings")
        .resizable(false)
        .show(context, |ui| {
            egui::ComboBox::from_label("Mode")
                .selected_text(format!("{gizmo_mode:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(gizmo_mode, GizmoMode::Rotate, "Rotate");
                    ui.selectable_value(gizmo_mode, GizmoMode::Translate, "Translate");
                    ui.selectable_value(gizmo_mode, GizmoMode::Scale, "Scale");
                });
            ui.end_row();

            egui::ComboBox::from_label("Orientation")
                .selected_text(format!("{gizmo_orientation:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        gizmo_orientation,
                        egui_gizmo::GizmoOrientation::Global,
                        "Global",
                    );
                    ui.selectable_value(
                        gizmo_orientation,
                        egui_gizmo::GizmoOrientation::Local,
                        "Local",
                    );
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
                egui::Label::new("X axis color").wrap(false).ui(ui);
            });

            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(ui, y_color, Alpha::Opaque);
                egui::Label::new("Y axis color").wrap(false).ui(ui);
            });
            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(ui, z_color, Alpha::Opaque);
                egui::Label::new("Z axis color").wrap(false).ui(ui);
            });
            ui.horizontal(|ui| {
                egui::color_picker::color_edit_button_srgba(ui, s_color, Alpha::Opaque);
                egui::Label::new("Screen axis color").wrap(false).ui(ui);
            });
            ui.end_row();
        });
    if *custom_highlight_color {
        visuals.highlight_color = Some(highlight_color);
    } else {
        visuals.highlight_color = None;
    }
}
