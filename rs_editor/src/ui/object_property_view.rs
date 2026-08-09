use crate::ui::component_edit::{ComponentEdit, UIComponentPropertyEvent};
use rs_content_manager::content_manager::ContentManager;
use rs_engine::{
    actor::Actor, components::component::Component, directional_light::DirectionalLight,
    engine::Engine, scene_node::*,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_localization::t;

pub struct ComponentEvent {
    pub component: SingleThreadMutType<Box<dyn Component>>,
    pub component_event: Box<dyn UIComponentPropertyEvent>,
}

pub enum EEventType {
    UpdateDirectionalLight(
        SingleThreadMutType<DirectionalLight>,
        f32,
        f32,
        f32,
        f32,
        f32,
    ),
    ChangeName(ESelectedObjectType, String),
    ComponentEvent(ComponentEvent),
}

#[derive(Clone)]
pub enum ESelectedObjectType {
    Actor(SingleThreadMutType<Actor>),
    DirectionalLight(SingleThreadMutType<DirectionalLight>),
    SceneNode(SingleThreadMutType<SceneNode>),
}

pub struct ObjectPropertyView {
    pub selected_object: Option<ESelectedObjectType>,
    pub materials: SingleThreadMutType<Vec<url::Url>>,
    pub animations: SingleThreadMutType<Vec<url::Url>>,
    pub static_meshes: SingleThreadMutType<Vec<url::Url>>,
}

impl ObjectPropertyView {
    pub fn new() -> ObjectPropertyView {
        ObjectPropertyView {
            selected_object: None,
            materials: SingleThreadMut::new(vec![]),
            animations: SingleThreadMut::new(vec![]),
            static_meshes: SingleThreadMut::new(vec![]),
        }
    }

    pub fn edit_name(name: &str, ui: &mut egui::Ui) -> Option<String> {
        let mut edit_name = name.to_string();
        let mut is_changed = false;
        ui.horizontal(|ui| {
            ui.label(t!("Name: ").as_ref());
            is_changed = ui.text_edit_singleline(&mut edit_name).changed();
        });
        if is_changed {
            return Some(edit_name);
        } else {
            None
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        component_edit: &mut ComponentEdit,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
    ) -> Option<EEventType> {
        let Some(selected_object) = self.selected_object.as_mut() else {
            return None;
        };
        let mut event = None;
        let selected_object_clone = selected_object.clone();
        match selected_object {
            ESelectedObjectType::Actor(actor) => {
                let actor = actor.borrow();
                ui.label(format!("{}", t!("Type: Actor")));
                if let Some(new_name) = Self::edit_name(&actor.name, ui) {
                    event = Some(EEventType::ChangeName(selected_object_clone, new_name));
                }
            }
            ESelectedObjectType::SceneNode(scene_node) => {
                let scene_node = scene_node.clone();
                let mut scene_node = scene_node.borrow_mut();
                {
                    let editable = component_edit.editable(scene_node.component_mut().as_mut());
                    if let Some(editable) = editable {
                        {
                            ui.label(editable.display_type_name());
                            let mut component = scene_node.component_mut();
                            let component = component.as_mut();
                            if let Some(new_name) = Self::edit_name(&component.get_name(), ui) {
                                component.set_name(new_name);
                            }
                            let mut transformation = component.get_transformation();
                            Self::transformation_widget_mut(&mut transformation, ui);
                            component.set_transformation(transformation);
                            Self::transformation_widget(&component.get_final_transformation(), ui);
                        }
                        let component_event = editable.edit(
                            ui,
                            scene_node.component_mut().as_mut(),
                            engine,
                            content_manager,
                            self,
                        );
                        if let Some(component_event) = component_event {
                            let component = scene_node.underlying_component();
                            event = Some(EEventType::ComponentEvent(ComponentEvent {
                                component,
                                component_event,
                            }))
                        }
                        return event;
                    }
                }
            }
            ESelectedObjectType::DirectionalLight(directional_light) => {
                ui.label(format!("{}", t!("Type: DirectionalLight")));
                let directional_light_clone = directional_light.clone();
                let mut component = directional_light.borrow_mut();
                if let Some(new_name) = Self::edit_name(&component.name, ui) {
                    event = Some(EEventType::ChangeName(selected_object_clone, new_name));
                }
                ui.checkbox(
                    &mut component.is_show_preview,
                    t!("Is show preview").as_ref(),
                );

                Self::transformation_widget_mut(component.get_transformation_mut(), ui);

                let mut is_changed = false;

                let mut left = component.left;
                is_changed = is_changed
                    || ui
                        .add(
                            egui::DragValue::new(&mut left)
                                .speed(0.1)
                                .prefix(t!("Left: ").as_ref()),
                        )
                        .changed();

                let mut right = component.right;
                is_changed = is_changed
                    || ui
                        .add(
                            egui::DragValue::new(&mut right)
                                .speed(0.1)
                                .prefix(t!("Right: ").as_ref()),
                        )
                        .changed();

                let mut top = component.top;
                is_changed = is_changed
                    || ui
                        .add(
                            egui::DragValue::new(&mut top)
                                .speed(0.1)
                                .prefix(t!("Top: ").as_ref()),
                        )
                        .changed();

                let mut bottom = component.bottom;
                is_changed = is_changed
                    || ui
                        .add(
                            egui::DragValue::new(&mut bottom)
                                .speed(0.1)
                                .prefix(t!("Bottom: ").as_ref()),
                        )
                        .changed();

                let mut far = component.far;
                is_changed = is_changed
                    || ui
                        .add(
                            egui::DragValue::new(&mut far)
                                .speed(0.1)
                                .prefix(t!("Far: ").as_ref()),
                        )
                        .changed();
                if is_changed {
                    event = Some(EEventType::UpdateDirectionalLight(
                        directional_light_clone,
                        left,
                        right,
                        top,
                        bottom,
                        far,
                    ));
                }
            }
        }

        event
    }

    pub fn transformation_widget(transformation: &glam::Mat4, ui: &mut egui::Ui) {
        let (scale, rotation, translation) = transformation.to_scale_rotation_translation();
        let rotation = glam::Vec3::from(rotation.to_euler(glam::EulerRot::XYZ));
        Self::affine_widget(&scale, &rotation, &translation, ui);
    }

    pub fn transformation_widget_mut(transformation: &mut glam::Mat4, ui: &mut egui::Ui) {
        let (mut scale, rotation, mut translation) = transformation.to_scale_rotation_translation();
        let mut rotation = glam::Vec3::from(rotation.to_euler(glam::EulerRot::XYZ));
        Self::affine_widget_mut(&mut scale, &mut rotation, &mut translation, ui);
        let rotation =
            glam::Quat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
        *transformation = glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);
    }

    pub fn affine_widget_mut(
        scale: &mut glam::Vec3,
        rotation: &mut glam::Vec3,
        translation: &mut glam::Vec3,
        ui: &mut egui::Ui,
    ) {
        ui.vertical(|ui| {
            Self::vec3_widget_mut(translation, ui, t!("Location"), true);
            Self::vec3_widget_mut(scale, ui, t!("Scale"), false);
            Self::vec3_widget_mut(rotation, ui, t!("Rotation"), true);
        });
        if translation.is_nan() {
            *translation = glam::Vec3::ZERO;
        }
        if scale.is_nan() {
            *scale = glam::Vec3::ONE;
        }
        if rotation.is_nan() {
            *rotation = glam::Vec3::ZERO;
        }
    }

    pub fn affine_widget(
        scale: &glam::Vec3,
        rotation: &glam::Vec3,
        translation: &glam::Vec3,
        ui: &mut egui::Ui,
    ) {
        ui.vertical(|ui| {
            Self::vec3_widget(translation, ui, t!("Location"));
            Self::vec3_widget(scale, ui, t!("Scale"));
            Self::vec3_widget(rotation, ui, t!("Rotation"));
        });
    }

    pub fn vec3_widget(value: &glam::Vec3, ui: &mut egui::Ui, label: impl AsRef<str>) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "{} x: {}, y: {}, z: {}",
                label.as_ref(),
                value.x,
                value.y,
                value.z
            ));
        });
    }

    pub fn vec3_widget_mut(
        value: &mut glam::Vec3,
        ui: &mut egui::Ui,
        label: impl AsRef<str>,
        is_allow_zero_value: bool,
    ) -> bool {
        let mut is_changed = false;
        let old = value.clone();
        ui.horizontal(|ui| {
            ui.label(label.as_ref());
            is_changed = is_changed
                || ui
                    .add(egui::DragValue::new(&mut value.x).speed(0.1).prefix("x: "))
                    .changed();
            is_changed = is_changed
                || ui
                    .add(egui::DragValue::new(&mut value.y).speed(0.1).prefix("y: "))
                    .changed();
            is_changed = is_changed
                || ui
                    .add(egui::DragValue::new(&mut value.z).speed(0.1).prefix("z: "))
                    .changed();
            if value.cmpeq(glam::Vec3::ZERO).any() && !is_allow_zero_value {
                *value = old;
            }
        });
        is_changed
    }

    pub fn vec1_widget_mut<Num: egui::emath::Numeric>(
        value: &mut Num,
        ui: &mut egui::Ui,
        label: impl AsRef<str>,
    ) -> bool {
        let mut is_changed = false;
        ui.horizontal(|ui| {
            ui.label(label.as_ref());
            is_changed = ui
                .add(egui::DragValue::new(value).speed(0.1).prefix("x: "))
                .changed();
        });
        is_changed
    }
}
