pub fn create_panel_ui_from_context(context: &egui::Context, id: Option<egui::Id>) -> egui::Ui {
    let mut panel_ui = egui::Ui::new(
        context.clone(),
        id.unwrap_or(egui::Id::new((context.viewport_id(), "__panel_ui"))),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(context.content_rect()),
    );
    panel_ui.set_clip_rect(context.content_rect());
    panel_ui
        .response()
        .widget_info(|| egui::WidgetInfo::new(egui::WidgetType::Panel));
    panel_ui
}
