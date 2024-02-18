use egui::{Context, Ui, Window};
use rs_core_minimal::settings::Settings;
use std::{cell::RefCell, rc::Rc};

pub fn draw(context: &Context, open: &mut bool, project_settings: Rc<RefCell<Settings>>) {
    Window::new("Project Settings")
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([350.0, 150.0])
        .show(context, |ui| {
            draw_content(ui, project_settings);
        });
}

fn draw_content(ui: &mut Ui, project_settings: Rc<RefCell<Settings>>) {
    ui.collapsing("Render", |ui| {
        ui.collapsing("Virtual Texture", |ui| {
            ui.vertical(|ui| {
                let mut project_settings = project_settings.borrow_mut();
                let render_setting = &mut project_settings.render_setting;
                ui.checkbox(
                    &mut render_setting.virtual_texture_setting.is_enable,
                    "Is Enable",
                );
                ui.add(
                    egui::DragValue::new(
                        &mut render_setting.virtual_texture_setting.feed_back_texture_div,
                    )
                    .speed(1)
                    .clamp_range(1..=10)
                    .prefix("Feed Back Texture Div:  "),
                )
            });
        });
    });
}
