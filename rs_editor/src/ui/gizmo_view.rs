use egui::Ui;
use egui_gizmo::{GizmoMode, GizmoOrientation, GizmoResult, GizmoVisuals};

pub struct GizmoView {
    pub visuals: GizmoVisuals,
    pub gizmo_mode: GizmoMode,
    pub gizmo_orientation: GizmoOrientation,
    pub custom_highlight_color: bool,

    view_matrix: glam::Mat4,
    projection_matrix: glam::Mat4,
    model_matrix: glam::Mat4,
}

impl GizmoView {
    pub fn default() -> GizmoView {
        GizmoView {
            visuals: Self::default_gizmo_visuals(),
            gizmo_mode: GizmoMode::Rotate,
            gizmo_orientation: GizmoOrientation::Global,
            custom_highlight_color: false,
            view_matrix: glam::Mat4::IDENTITY,
            projection_matrix: glam::Mat4::IDENTITY,
            model_matrix: glam::Mat4::IDENTITY,
        }
    }

    pub fn draw(
        &mut self,
        context: &egui::Context,
        view_matrix: glam::Mat4,
        projection_matrix: glam::Mat4,
        model_matrix: glam::Mat4,
    ) -> Option<GizmoResult> {
        self.view_matrix = view_matrix;
        self.projection_matrix = projection_matrix;
        self.model_matrix = model_matrix;
        let mut gizmo_response: Option<GizmoResult> = None;
        egui::Area::new("Gizmo Viewport".into())
            .fixed_pos((0.0, 0.0))
            .show(context, |ui| {
                ui.with_layer_id(egui::LayerId::background(), |ui| {
                    gizmo_response = self.interact(ui)
                });
            });
        gizmo_response
    }

    fn default_gizmo_visuals() -> GizmoVisuals {
        let stroke_width = 4.0;
        let gizmo_size = 75.0;
        let custom_highlight_color = false;
        let highlight_color = egui::Color32::GOLD;
        let x_color = egui::Color32::from_rgb(255, 0, 148);
        let y_color = egui::Color32::from_rgb(148, 255, 0);
        let z_color = egui::Color32::from_rgb(0, 148, 255);
        let s_color = egui::Color32::WHITE;
        let inactive_alpha = 0.5;
        let highlight_alpha = 1.0;

        let visuals = GizmoVisuals {
            x_color,
            y_color,
            z_color,
            s_color,
            inactive_alpha,
            highlight_alpha,
            highlight_color: if custom_highlight_color {
                Some(highlight_color)
            } else {
                None
            },
            stroke_width,
            gizmo_size,
        };
        visuals
    }

    pub fn interact(&mut self, ui: &mut Ui) -> Option<GizmoResult> {
        let gizmo = egui_gizmo::Gizmo::new("Gizmo")
            .view_matrix(self.view_matrix.to_cols_array_2d().into())
            .projection_matrix(self.projection_matrix.to_cols_array_2d().into())
            .model_matrix(self.model_matrix.to_cols_array_2d().into())
            .mode(self.gizmo_mode)
            .orientation(self.gizmo_orientation)
            .snapping(false)
            .snap_angle(0.0f32)
            .snap_distance(0.0f32)
            .visuals(self.visuals);
        let last_gizmo_response = gizmo.interact(ui);
        if let Some(gizmo_response) = last_gizmo_response {
            Self::show_gizmo_status(ui, &gizmo_response);
            Some(gizmo_response)
        } else {
            None
        }
    }

    fn show_gizmo_status(ui: &egui::Ui, response: &egui_gizmo::GizmoResult) {
        let length = glam::Vec3::from(response.value).length();

        let text = match response.mode {
            GizmoMode::Rotate => format!("{:.1}°, {:.2} rad", length.to_degrees(), length),

            GizmoMode::Translate | GizmoMode::Scale => format!(
                "dX: {:.2}, dY: {:.2}, dZ: {:.2}",
                response.value[0], response.value[1], response.value[2]
            ),
        };

        let rect = ui.clip_rect();
        ui.painter().text(
            egui::pos2(rect.left() + 10.0, rect.bottom() - 10.0),
            egui::Align2::LEFT_BOTTOM,
            text,
            egui::FontId::default(),
            egui::Color32::WHITE,
        );
    }
}
