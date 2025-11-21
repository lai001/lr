use egui::{Color32, Ui};
use egui_plot::{Line, MarkerShape, Plot, PlotPoints, Points};
use rs_engine::content::curve::{ControlPoint, Curve};
use uniform_cubic_splines::{basis::*, *};

pub struct CurveViewDataSource {
    pub is_drag_enable: bool,
    pub hovered_plot_item: Option<egui::Id>,
    pub last_hover_plot_pos: Option<glam::DVec2>,
    point_radius: f32,
    point_color: Color32,
    line_color: Color32,
    line_points: usize,
}

impl Default for CurveViewDataSource {
    fn default() -> Self {
        Self {
            is_drag_enable: true,
            hovered_plot_item: None,
            last_hover_plot_pos: None,
            point_radius: 3.5,
            point_color: Color32::BLUE,
            line_color: Color32::RED,
            line_points: 256,
        }
    }
}

pub fn draw(opend_curve: &mut Curve, ui: &mut Ui, data_source: &mut CurveViewDataSource) {
    let plot = Plot::new("Curve")
        .allow_drag(data_source.is_drag_enable)
        .allow_boxed_zoom(false)
        .allow_zoom(true)
        .allow_scroll(false)
        .allow_double_click_reset(false)
        .data_aspect(1.0);
    let plot_response = plot.show(ui, |plot_ui| {
        for line in lines(
            &opend_curve.control_points,
            data_source.line_color,
            data_source.line_points,
        ) {
            plot_ui.line(line);
        }
        for control_point in control_points_ui(
            &opend_curve.control_points,
            data_source.point_radius,
            data_source.point_color,
        ) {
            plot_ui.points(control_point);
        }
    });

    let plot_pos = if let Some(hover_pos) = plot_response.response.hover_pos() {
        let plot_pos = plot_response.transform.value_from_position(hover_pos);
        Some(glam::dvec2(plot_pos.x, plot_pos.y))
    } else {
        None
    };

    let is_pointer_button_down_on = plot_response.response.is_pointer_button_down_on();

    plot_response.response.context_menu(|ui| {
        let response = ui.button("Add");
        if response.clicked() {
            if let Some(last_hover_plot_pos) = plot_pos.or(data_source.last_hover_plot_pos) {
                let index = opend_curve.control_points.len();
                opend_curve
                    .control_points
                    .push(ControlPoint::new(index, last_hover_plot_pos));
                opend_curve.sort_by_x();
            }
            ui.close_kind(egui::UiKind::Menu);
        }
    });

    if is_pointer_button_down_on && plot_response.hovered_plot_item.is_some() {
        data_source.hovered_plot_item = plot_response.hovered_plot_item;
    }

    if is_pointer_button_down_on {
        if let Some(hovered_plot_item_id) = data_source.hovered_plot_item {
            let mut is_need_sort = false;
            for control_point in opend_curve.control_points.iter_mut() {
                if egui::Id::new(control_point.id.clone()) == hovered_plot_item_id {
                    if let Some(last_hover_plot_pos) = plot_pos.or(data_source.last_hover_plot_pos)
                    {
                        control_point.position = last_hover_plot_pos;
                        is_need_sort = true;
                    }
                }
            }
            if is_need_sort {
                opend_curve.sort_by_x();
            }
        }
    } else {
        if data_source.hovered_plot_item.is_some() {
            data_source.hovered_plot_item = None;
        }
    }

    data_source.is_drag_enable = data_source.hovered_plot_item.is_none();

    if let Some(plot_pos) = plot_pos {
        data_source.last_hover_plot_pos = Some(plot_pos);
    }
}

fn control_points_ui(
    control_points: &[ControlPoint],
    radius: f32,
    color: Color32,
) -> Vec<Points<'_>> {
    let mut points = Vec::with_capacity(control_points.len());
    for control_point in control_points {
        points.push(
            Points::new(
                "",
                vec![[control_point.position.x, control_point.position.y]],
            )
            .shape(MarkerShape::Circle)
            .radius(radius)
            .color(color)
            .id(control_point.id.clone()),
        );
    }
    points
}

fn lines(control_points: &[ControlPoint], color: Color32, points: usize) -> Vec<Line<'_>> {
    if control_points.len() >= 2 {
        let mut knot_spacing: Vec<f64> = Vec::with_capacity(2 + control_points.len());
        knot_spacing.push(control_points[0].position.x);
        for item in control_points {
            knot_spacing.push(item.position.x);
        }
        knot_spacing.push(control_points.last().unwrap().position.x);

        let mut knots: Vec<f64> = Vec::with_capacity(2 + control_points.len());
        knots.push(control_points[0].position.y);
        for item in control_points {
            knots.push(item.position.y);
        }
        knots.push(control_points.last().unwrap().position.y);

        let mut lines = Vec::with_capacity(1);
        lines.push(
            Line::new(
                "",
                PlotPoints::from_explicit_callback(
                    move |x| {
                        let v = spline_inverse::<CatmullRom, _>(x, &knot_spacing).unwrap();
                        let y = spline::<CatmullRom, _, _>(v, &knots);
                        y
                    },
                    ..,
                    points,
                ),
            )
            .allow_hover(false)
            .color(color),
        );
        lines
    } else {
        vec![]
    }
}
