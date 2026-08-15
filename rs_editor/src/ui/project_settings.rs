use crate::ui::misc::ToUIString;
use egui::{Context, Ui};
use rs_core_minimal::{
    settings::{Backends, EAntialiasType, PowerPreference, Settings},
    types::HasUrl,
};
use rs_engine::content::content_file_type::EContentFileType;
use rs_foundation::new::SingleThreadMutType;
use rs_localization::{self, t};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub enum EEventType {
    AntialiasType(EAntialiasType),
    SetLocale(String),
}

pub fn draw(
    window: egui::Window,
    context: &Context,
    open: &mut bool,
    project_settings: Rc<RefCell<Settings>>,
    contents: SingleThreadMutType<Vec<EContentFileType>>,
) -> Option<EEventType> {
    let mut event: Option<EEventType> = None;
    window
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([350.0, 150.0])
        .show(context, |ui| {
            event = draw_content(ui, project_settings, contents);
        });
    event
}

fn locale_name(locale: &str) -> String {
    match locale {
        "en" => "English".to_string(),
        "zh-CN" => "中文(简体)".to_string(),
        other => other.to_string(),
    }
}

fn draw_content(
    ui: &mut Ui,
    project_settings: Rc<RefCell<Settings>>,
    contents: SingleThreadMutType<Vec<EContentFileType>>,
) -> Option<EEventType> {
    let mut event: Option<EEventType> = None;
    ui.collapsing(t!("Editor"), |ui| {
        let mut project_settings = project_settings.borrow_mut();
        let auto_open_last_project =
            &mut project_settings.editor_settings.is_auto_open_last_project;
        ui.checkbox(auto_open_last_project, t!("Is auto open last project"));

        let locales = rs_localization::available_locales();
        let mut locale = project_settings.editor_settings.locale.clone();
        let current_text = if locale.is_empty() {
            t!("None").to_string()
        } else {
            locale_name(&locale)
        };
        egui::ComboBox::from_label(t!("Language"))
            .selected_text(current_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut locale, String::new(), t!("None"))
                    .clicked()
                {
                    project_settings.editor_settings.locale = locale.clone();
                    event = Some(EEventType::SetLocale(locale.clone()));
                }
                for available_locale in locales {
                    let name = locale_name(&available_locale);
                    if ui
                        .selectable_value(&mut locale, available_locale.to_string(), name)
                        .clicked()
                    {
                        project_settings.editor_settings.locale = locale.clone();
                        event = Some(EEventType::SetLocale(locale.clone()));
                    }
                }
            });
    });
    ui.collapsing(t!("Engine"), |ui| {
        let mut project_settings = project_settings.borrow_mut();
        let engine_settings = &mut project_settings.engine_settings;
        let contents = contents.borrow();
        let urls: Vec<url::Url> = contents
            .iter()
            .map(|x| {
                if let Some(level) = x
                    .borrow()
                    .downcast_ref::<rs_engine::content::level::Level>()
                {
                    Some(level.get_url())
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        egui::ComboBox::from_label(t!("Default Level"))
            .selected_text(format!(
                "{}",
                engine_settings
                    .default_level
                    .as_ref()
                    .map(|x| x.to_ui_string())
                    .unwrap_or(t!("None").to_string())
            ))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                ui.set_min_width(60.0);
                for url in urls {
                    let text = format!("{}", &url.to_ui_string());
                    ui.selectable_value(&mut engine_settings.default_level, Some(url), text);
                }
            });
    });
    ui.collapsing(t!("Render"), |ui| {
        let mut project_settings = project_settings.borrow_mut();
        let render_setting = &mut project_settings.render_setting;
        ui.collapsing(t!("Virtual Texture"), |ui| {
            ui.vertical(|ui| {
                ui.checkbox(
                    &mut render_setting.virtual_texture_setting.is_enable,
                    t!("Is Enable"),
                );
                ui.add(
                    egui::DragValue::new(
                        &mut render_setting.virtual_texture_setting.feed_back_texture_div,
                    )
                    .speed(1)
                    .range(1..=10)
                    .prefix(t!("Feed Back Texture Div:  ")),
                );

                let backends = &mut render_setting.backends;
                egui::ComboBox::from_label(t!("Select Backends"))
                    .selected_text(format!("{:?}", backends))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        ui.set_min_width(60.0);
                        ui.selectable_value(backends, Backends::DX12, "DX12");
                        ui.selectable_value(backends, Backends::GL, "GL");
                        ui.selectable_value(backends, Backends::Vulkan, "Vulkan");
                        ui.selectable_value(backends, Backends::Primary, t!("Primary"));
                    });

                egui::ComboBox::from_label(t!("Select Power Preference"))
                    .selected_text(format!("{:?}", render_setting.power_preference))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        ui.set_min_width(60.0);
                        ui.selectable_value(
                            &mut render_setting.power_preference,
                            PowerPreference::None,
                            t!("None"),
                        );
                        ui.selectable_value(
                            &mut render_setting.power_preference,
                            PowerPreference::HighPerformance,
                            t!("HighPerformance"),
                        );
                        ui.selectable_value(
                            &mut render_setting.power_preference,
                            PowerPreference::LowPower,
                            t!("LowPower"),
                        );
                    });
            });
        });
        egui::ComboBox::from_label(t!("Antialias Type"))
            .selected_text(format!("{:?}", render_setting.antialias_type))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
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
