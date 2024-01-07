use crate::data_source::DataSource;
use egui::*;

#[derive(Default, Debug)]
pub struct ClickEvent {
    pub is_open_project: bool,
    pub is_new_project: bool,
    pub is_import_asset: bool,
}

pub struct EditorUI {}

impl EditorUI {
    pub fn build(context: &Context, data_source: &mut DataSource) -> ClickEvent {
        let mut click = ClickEvent::default();
        TopBottomPanel::top("menu_bar").show(context, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.set_min_width(220.0);
                    ui.style_mut().wrap = Some(false);
                    if ui.add(Button::new("New Project")).clicked() {
                        data_source.is_new_project_window_open = true;
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Open Project")).clicked() {
                        click.is_open_project = true;
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Import Asset")).clicked() {
                        click.is_import_asset = true;
                        ui.close_menu();
                    }
                });
            });
        });

        let mut is_new_project_window_open = data_source.is_new_project_window_open;
        Window::new("New Project")
            .open(&mut is_new_project_window_open)
            .show(context, |ui| {
                ui.text_edit_singleline(&mut data_source.new_project_name);
                
                if ui.add(Button::new("OK")).clicked() {
                    click.is_new_project = true;
                    data_source.is_new_project_window_open = false;
                }
            });
        data_source.is_new_project_window_open = is_new_project_window_open;
        click
    }
}
