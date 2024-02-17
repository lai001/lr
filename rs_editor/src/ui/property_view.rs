use super::texture_property_view;
use egui::{Context, Ui, Vec2, Window};
use rs_artifact::property_value_type::EPropertyValueType;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub enum ESelectedObject {
    Node(Rc<RefCell<crate::level::Node>>),
    TextureFile(Rc<RefCell<crate::texture::TextureFile>>),
}

pub struct DataSource {
    pub is_open: bool,
    pub selected_node: Option<Rc<RefCell<crate::level::Node>>>,
    pub selected_object: Option<ESelectedObject>,
}

impl DataSource {
    pub fn new() -> Self {
        Self {
            is_open: true,
            selected_node: None,
            selected_object: None,
        }
    }
}

#[derive(Debug)]
pub enum EValueModifierType {
    ValueType(EPropertyValueType),
    Assign,
}

#[derive(Debug)]
pub enum EClickEventType {
    Node(HashMap<String, EValueModifierType>),
    TextureFile(crate::ui::texture_property_view::EClickEventType),
}

pub fn draw(
    context: &Context,
    open: &mut bool,
    selected_object: &mut Option<ESelectedObject>,
) -> Option<EClickEventType> {
    let name = {
        if let Some(selected_object) = selected_object {
            match selected_object {
                ESelectedObject::Node(node) => node.borrow().name.to_string(),
                ESelectedObject::TextureFile(texture_file) => {
                    texture_file.borrow().name.to_string()
                }
            }
        } else {
            "".to_string()
        }
    };
    let mut click: Option<EClickEventType> = None;
    Window::new(format!("Property ({})", name))
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([250.0, 500.0])
        .show(context, |ui| {
            if let Some(selected_object) = selected_object {
                match selected_object {
                    ESelectedObject::Node(node) => {
                        egui::Grid::new("PropertyGrid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                let value_changed = content(ui, &mut node.borrow_mut());
                                click = Some(EClickEventType::Node(value_changed));
                            });
                    }
                    ESelectedObject::TextureFile(texture_file) => {
                        let click_event = texture_property_view::draw(ui, texture_file.clone());
                        click = click_event.map_or(None, |x| Some(EClickEventType::TextureFile(x)));
                    }
                }
            }
        });
    click
}

fn content(
    ui: &mut Ui,
    selected_node: &mut crate::level::Node,
) -> HashMap<String, EValueModifierType> {
    let mut value_changed: HashMap<String, EValueModifierType> = HashMap::new();

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
            EPropertyValueType::Vec2(vec2) => {
                if ui
                    .add(egui::DragValue::new(&mut vec2.x).speed(0.01).prefix("x: "))
                    .changed()
                {}
                if ui
                    .add(egui::DragValue::new(&mut vec2.y).speed(0.01).prefix("y: "))
                    .changed()
                {}
            }
            EPropertyValueType::Vec3(vec3) => {
                if ui
                    .add(egui::DragValue::new(&mut vec3.x).speed(0.01).prefix("x: "))
                    .changed()
                {}
                if ui
                    .add(egui::DragValue::new(&mut vec3.y).speed(0.01).prefix("y: "))
                    .changed()
                {}
                if ui
                    .add(egui::DragValue::new(&mut vec3.z).speed(0.01).prefix("z: "))
                    .changed()
                {}
            }
            EPropertyValueType::Quat(quat) => {
                if ui
                    .add(egui::DragValue::new(&mut quat.x).speed(0.01).prefix("x: "))
                    .changed()
                {}
                if ui
                    .add(egui::DragValue::new(&mut quat.y).speed(0.01).prefix("y: "))
                    .changed()
                {}
                if ui
                    .add(egui::DragValue::new(&mut quat.z).speed(0.01).prefix("z: "))
                    .changed()
                {}
                if ui
                    .add(egui::DragValue::new(&mut quat.w).speed(0.01).prefix("w: "))
                    .changed()
                {}
            }
        }
        ui.end_row();
    }

    value_changed
}
