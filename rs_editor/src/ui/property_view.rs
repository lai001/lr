use super::texture_property_view;
use egui::{Context, Ui, Vec2};
use rs_artifact::property_value_type::EPropertyValueType;
use rs_engine::content::texture::TextureFile;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub enum ESelectedObject {
    Actor(Rc<RefCell<rs_engine::actor::Actor>>),
    TextureFile(Rc<RefCell<TextureFile>>),
}

pub struct DataSource {
    pub is_open: bool,
    pub selected_actor: Option<Rc<RefCell<rs_engine::actor::Actor>>>,
    pub selected_object: Option<ESelectedObject>,
}

impl DataSource {
    pub fn new() -> Self {
        Self {
            is_open: true,
            selected_actor: None,
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
    window: egui::Window,
    context: &Context,
    open: &mut bool,
    selected_object: &mut Option<ESelectedObject>,
) -> Option<EClickEventType> {
    let mut click: Option<EClickEventType> = None;
    window
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([250.0, 500.0])
        .show(context, |ui| {
            if let Some(selected_object) = selected_object {
                match selected_object {
                    ESelectedObject::TextureFile(texture_file) => {
                        let click_event = texture_property_view::draw(ui, texture_file.clone());
                        click = click_event.map_or(None, |x| Some(EClickEventType::TextureFile(x)));
                    }
                    ESelectedObject::Actor(actor) => todo!(),
                }
            }
        });
    click
}

fn content(
    ui: &mut Ui,
    name: &str,
    values: Rc<RefCell<HashMap<String, EPropertyValueType>>>,
) -> HashMap<String, EValueModifierType> {
    let mut value_changed: HashMap<String, EValueModifierType> = HashMap::new();
    ui.label(format!("name: {}", name));
    ui.end_row();

    for (property_name, property_value) in values.borrow_mut().iter_mut() {
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
