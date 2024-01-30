use egui::{Context, Ui, Vec2, Window};
use rs_artifact::property_value_type::EPropertyValueType;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct DataSource {
    pub is_open: bool,
    pub selected_node: Option<Rc<RefCell<crate::level::Node>>>,
}

impl DataSource {
    pub fn new() -> Self {
        Self {
            is_open: true,
            selected_node: None,
        }
    }
}

#[derive(Debug)]
pub enum EValueModifierType {
    ValueType(EPropertyValueType),
    Assign,
}

pub fn draw(
    context: &Context,
    open: &mut bool,
    selected_node: Option<&mut crate::level::Node>,
) -> HashMap<String, EValueModifierType> {
    let mut name = "".to_string();
    if let Some(selected_node) = &selected_node {
        name = selected_node.name.clone();
    }
    let mut value_changed: HashMap<String, EValueModifierType> = HashMap::new();
    Window::new(format!("Property ({})", name))
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
                        value_changed = content(ui, selected_node);
                    });
                ui.allocate_space(ui.available_size());
            });
        });
    value_changed
}

fn content(
    ui: &mut Ui,
    selected_node: Option<&mut crate::level::Node>,
) -> HashMap<String, EValueModifierType> {
    let mut value_changed: HashMap<String, EValueModifierType> = HashMap::new();

    if let Some(selected_node) = selected_node {
        for (property_name, property_value) in &mut selected_node.values {
            ui.label(property_name.clone());
            match property_value {
                EPropertyValueType::Texture(_) => {
                    let button = egui::Button::image(egui::include_image!(
                        "../../../Resource/Editor/circular.svg"
                    ))
                    .min_size(Vec2::splat(20.0));
                    let response = ui.add_sized(Vec2::splat(20.0), button);
                    if response.clicked() {
                        value_changed.insert(property_name.clone(), EValueModifierType::Assign);
                    }
                }
                EPropertyValueType::Int(scalar) => {
                    if ui.add(egui::DragValue::new(scalar).speed(1)).changed() {}
                }
                EPropertyValueType::Float(scalar) => {
                    if ui.add(egui::DragValue::new(scalar).speed(0.01)).changed() {}
                }
                EPropertyValueType::String(string) => {
                    if ui.add(egui::TextEdit::singleline(string)).changed() {}
                }
            }
            ui.end_row();
        }
    }

    value_changed
}
