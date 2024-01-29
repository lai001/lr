use egui::{Context, Vec2, Window};
use rs_artifact::property_value_type::EPropertyValueType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct DataSource {
    pub is_open: bool,
    pub values: HashMap<String, PropertyValue>,
}

impl DataSource {
    pub fn new() -> Self {
        let values = HashMap::new();
        Self {
            is_open: false,
            values,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropertyValue {
    pub value_type: EPropertyValueType,
    pub is_changed: bool,
}

pub fn draw(
    context: &Context,
    open: &mut bool,
    property_values: &mut HashMap<String, PropertyValue>,
) {
    Window::new("Property")
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([250.0, 500.0])
        .show(context, |ui| {
            ui.vertical(|ui| {
                ui.set_max_height(500.0);
                ui.set_max_width(250.0);
                egui::Grid::new("PropertyGrid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for (property_name, property_value) in property_values {
                            ui.label(property_name.clone());
                            match &mut property_value.value_type {
                                EPropertyValueType::Texture(_) => {
                                    ui.add_sized(
                                        Vec2::splat(20.0),
                                        egui::Button::image(egui::include_image!(
                                            "../../../Resource/Editor/circular.svg"
                                        ))
                                        .min_size(Vec2::splat(20.0)),
                                    );
                                }
                                EPropertyValueType::Int(scalar) => {
                                    if ui.add(egui::DragValue::new(scalar).speed(1)).changed() {
                                        property_value.is_changed = true;
                                    }
                                }
                                EPropertyValueType::Float(scalar) => {
                                    if ui.add(egui::DragValue::new(scalar).speed(0.01)).changed() {
                                        property_value.is_changed = true;
                                    }
                                }
                                EPropertyValueType::String(string) => {
                                    if ui.add(egui::TextEdit::singleline(string)).changed() {
                                        property_value.is_changed = true;
                                    }
                                }
                            }
                            ui.end_row();
                        }
                    });
                ui.allocate_space(ui.available_size());
            });
        });
}
