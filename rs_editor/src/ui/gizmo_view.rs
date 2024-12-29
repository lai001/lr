use egui::Ui;
use transform_gizmo_egui::{math::Transform, *};

pub struct GizmoView {
    gizmo: Gizmo,
    pub visuals: GizmoVisuals,
    pub gizmo_mode: EnumSet<GizmoMode>,
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
            gizmo_mode: GizmoMode::all_rotate(),
            gizmo_orientation: GizmoOrientation::Global,
            custom_highlight_color: false,
            view_matrix: glam::Mat4::IDENTITY,
            projection_matrix: glam::Mat4::IDENTITY,
            model_matrix: glam::Mat4::IDENTITY,
            gizmo: Gizmo::default(),
        }
    }

    pub fn draw(
        &mut self,
        context: &egui::Context,
        view_matrix: glam::Mat4,
        projection_matrix: glam::Mat4,
        model_matrix: glam::Mat4,
    ) -> Option<(GizmoResult, Vec<Transform>)> {
        self.view_matrix = view_matrix;
        self.projection_matrix = projection_matrix;
        self.model_matrix = model_matrix;
        let mut gizmo_response = None;
        egui::Area::new("Gizmo Viewport".into())
            .fixed_pos((0.0, 0.0))
            .show(context, |ui| {
                ui.scope_builder(
                    egui::UiBuilder::new().layer_id(egui::LayerId::background()),
                    |ui| gizmo_response = self.interact(ui),
                );
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

    fn interact(&mut self, ui: &mut Ui) -> Option<(GizmoResult, Vec<Transform>)> {
        let viewport = ui.clip_rect();
        let snapping = ui.input(|input| input.modifiers.ctrl);
        self.gizmo.update_config(GizmoConfig {
            view_matrix: self.view_matrix.as_dmat4().into(),
            projection_matrix: self.projection_matrix.as_dmat4().into(),
            viewport,
            modes: self.gizmo_mode.into(),
            orientation: self.gizmo_orientation,
            snapping,
            visuals: self.visuals,
            ..Default::default()
        });

        let (scale, rotation, translation) = self.model_matrix.to_scale_rotation_translation();
        let (scale, rotation, translation) = (
            scale.as_dvec3(),
            rotation.as_dquat(),
            translation.as_dvec3(),
        );
        let transform = Transform::from_scale_rotation_translation(scale, rotation, translation);

        let last_gizmo_response = self.gizmo.interact(ui, &[transform]);
        if let Some((gizmo_result, r_transform)) = last_gizmo_response {
            Self::show_gizmo_status(ui, &gizmo_result);
            Some((gizmo_result, r_transform))
        } else {
            None
        }
    }

    pub fn is_focused(&self) -> bool {
        self.gizmo.is_focused()
    }

    fn show_gizmo_status(ui: &egui::Ui, result: &GizmoResult) {
        let text = match result {
            GizmoResult::Rotation {
                axis,
                delta: _,
                total,
                is_view_axis: _,
            } => {
                format!(
                    "Rotation axis: ({:.2}, {:.2}, {:.2}), Angle: {:.2} deg",
                    axis.x,
                    axis.y,
                    axis.z,
                    total.to_degrees()
                )
            }
            GizmoResult::Translation { delta: _, total } => {
                format!(
                    "Translation: ({:.2}, {:.2}, {:.2})",
                    total.x, total.y, total.z,
                )
            }
            GizmoResult::Scale { total } => {
                format!("Scale: ({:.2}, {:.2}, {:.2})", total.x, total.y, total.z,)
            }
            GizmoResult::Arcball { delta: _, total } => {
                let (axis, angle) =
                    glam::dquat(total.v.x, total.v.y, total.v.z, total.s).to_axis_angle();
                format!(
                    "Rotation axis: ({:.2}, {:.2}, {:.2}), Angle: {:.2} deg",
                    axis.x,
                    axis.y,
                    axis.z,
                    angle.to_degrees()
                )
            }
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
