use egui::{Context, Ui};
use rs_core_minimal::settings::{Backends, EAntialiasType, PowerPreference, Settings};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub enum EEventType {
    AntialiasType(EAntialiasType),
}

pub fn draw(
    window: egui::Window,
    context: &Context,
    open: &mut bool,
    project_settings: Rc<RefCell<Settings>>,
) -> Option<EEventType> {
    let mut event: Option<EEventType> = None;
    window
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([350.0, 150.0])
        .show(context, |ui| {
            event = draw_content(ui, project_settings);
        });
    event
}

fn draw_content(ui: &mut Ui, project_settings: Rc<RefCell<Settings>>) -> Option<EEventType> {
    let mut event: Option<EEventType> = None;
    ui.collapsing("Render", |ui| {
        let mut project_settings = project_settings.borrow_mut();
        let render_setting = &mut project_settings.render_setting;
        ui.collapsing("Virtual Texture", |ui| {
            ui.vertical(|ui| {
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
                );

                let backends = &mut render_setting.backends;
                egui::ComboBox::from_label("Select Backends")
                    .selected_text(format!("{:?}", backends))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        ui.selectable_value(backends, Backends::DX12, "DX12");
                        ui.selectable_value(backends, Backends::GL, "GL");
                        ui.selectable_value(backends, Backends::Vulkan, "Vulkan");
                        ui.selectable_value(backends, Backends::Primary, "Primary");
                    });

                egui::ComboBox::from_label("Select Power Preference")
                    .selected_text(format!("{:?}", render_setting.power_preference))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        ui.selectable_value(
                            &mut render_setting.power_preference,
                            PowerPreference::None,
                            "None",
                        );
                        ui.selectable_value(
                            &mut render_setting.power_preference,
                            PowerPreference::HighPerformance,
                            "HighPerformance",
                        );
                        ui.selectable_value(
                            &mut render_setting.power_preference,
                            PowerPreference::LowPower,
                            "LowPower",
                        );
                    });
            });
        });
        egui::ComboBox::from_label("Antialias Type")
            .selected_text(format!("{:?}", render_setting.antialias_type))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                for ty in [
                    EAntialiasType::None,
                    EAntialiasType::FXAA,
                    EAntialiasType::MSAA,
                ] {
                    if ui
                        .selectable_value(
                            &mut render_setting.antialias_type,
                            ty.clone(),
                            format!("{:?}", ty),
                        )
                        .clicked()
                    {
                        event = Some(EEventType::AntialiasType(ty.clone()));
                    }
                }
            });
    });
    event
}
