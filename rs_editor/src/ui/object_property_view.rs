use rs_engine::{
    actor::Actor, content::level::DirectionalLight, scene_node::*,
    static_mesh_component::StaticMeshComponent,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};

pub struct UpdateMaterial {
    pub selected_object: ESelectedObjectType,
    pub old: Option<url::Url>,
    pub new: Option<url::Url>,
}

pub enum EEventType {
    UpdateMaterial(UpdateMaterial),
}

#[derive(Clone)]
pub enum ESelectedObjectType {
    Actor(SingleThreadMutType<Actor>),
    SceneComponent(SingleThreadMutType<SceneComponent>),
    StaticMeshComponent(SingleThreadMutType<StaticMeshComponent>),
    SkeletonMeshComponent(SingleThreadMutType<SkeletonMeshComponent>),
    DirectionalLight(SingleThreadMutType<DirectionalLight>),
}

pub struct ObjectPropertyView {
    pub selected_object: Option<ESelectedObjectType>,
    pub materials: SingleThreadMutType<Vec<url::Url>>,
}

impl ObjectPropertyView {
    pub fn new() -> ObjectPropertyView {
        ObjectPropertyView {
            selected_object: None,
            materials: SingleThreadMut::new(vec![]),
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) -> Option<EEventType> {
        let Some(selected_object) = self.selected_object.as_mut() else {
            return None;
        };
        let mut event = None;
        let selected_object_clone = selected_object.clone();
        match selected_object {
            ESelectedObjectType::Actor(actor) => {
                ui.label(actor.borrow().name.clone());
            }
            ESelectedObjectType::SceneComponent(scene_component) => {
                let mut component = scene_component.borrow_mut();
                ui.label(component.name.clone());
                let (mut scale, rotation, mut translation) =
                    component.transformation.to_scale_rotation_translation();
                let mut rotation = glam::Vec3::from(rotation.to_euler(glam::EulerRot::XYZ));
                Self::transformation_detail(&mut scale, &mut rotation, &mut translation, ui);
                let rotation =
                    glam::Quat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
                component.transformation =
                    glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);
            }
            ESelectedObjectType::StaticMeshComponent(static_mesh_component) => {
                let mut component = static_mesh_component.borrow_mut();
                ui.label(component.name.clone());
                let (mut scale, rotation, mut translation) =
                    component.transformation.to_scale_rotation_translation();
                let mut rotation = glam::Vec3::from(rotation.to_euler(glam::EulerRot::XYZ));
                Self::transformation_detail(&mut scale, &mut rotation, &mut translation, ui);
                let rotation =
                    glam::Quat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
                component.transformation =
                    glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);

                egui::ComboBox::from_label("Material")
                    .selected_text(format!("{}", {
                        match &component.material_url {
                            Some(material_url) => material_url.to_string(),
                            None => "None".to_string(),
                        }
                    }))
                    .show_ui(ui, |ui| {
                        let mut collection: Vec<Option<url::Url>> = vec![];
                        collection.push(None);
                        collection.append(
                            &mut self
                                .materials
                                .borrow()
                                .iter()
                                .map(|x| Some(x.clone()))
                                .collect(),
                        );

                        for material in collection {
                            let old = component.material_url.clone();
                            let text = material
                                .as_ref()
                                .map(|x| x.to_string())
                                .unwrap_or("None".to_string());
                            let is_changed = ui
                                .selectable_value(
                                    &mut component.material_url,
                                    material.clone(),
                                    text,
                                )
                                .changed();
                            if is_changed {
                                event = Some(EEventType::UpdateMaterial(UpdateMaterial {
                                    selected_object: selected_object_clone.clone(),
                                    old,
                                    new: material.clone(),
                                }));
                            }
                        }
                    });
            }
            ESelectedObjectType::SkeletonMeshComponent(skeleton_mesh_component) => {
                let mut component = skeleton_mesh_component.borrow_mut();
                ui.label(component.name.clone());
                let (mut scale, rotation, mut translation) =
                    component.transformation.to_scale_rotation_translation();
                let mut rotation = glam::Vec3::from(rotation.to_euler(glam::EulerRot::XYZ));
                Self::transformation_detail(&mut scale, &mut rotation, &mut translation, ui);
                let rotation =
                    glam::Quat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
                component.transformation =
                    glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);

                egui::ComboBox::from_label("Material")
                    .selected_text(format!("{}", {
                        match &component.material_url {
                            Some(material_url) => material_url.to_string(),
                            None => "None".to_string(),
                        }
                    }))
                    .show_ui(ui, |ui| {
                        let mut collection: Vec<Option<url::Url>> = vec![];
                        collection.push(None);
                        collection.append(
                            &mut self
                                .materials
                                .borrow()
                                .iter()
                                .map(|x| Some(x.clone()))
                                .collect(),
                        );

                        for material in collection {
                            let old = component.material_url.clone();
                            let text = material
                                .as_ref()
                                .map(|x| x.to_string())
                                .unwrap_or("None".to_string());
                            let is_changed = ui
                                .selectable_value(
                                    &mut component.material_url,
                                    material.clone(),
                                    text,
                                )
                                .changed();
                            if is_changed {
                                event = Some(EEventType::UpdateMaterial(UpdateMaterial {
                                    selected_object: selected_object_clone.clone(),
                                    old,
                                    new: material.clone(),
                                }));
                            }
                        }
                    });
            }
            ESelectedObjectType::DirectionalLight(directional_light) => {
                let mut component = directional_light.borrow_mut();
                let (mut scale, rotation, mut translation) = component
                    .get_interactive_transformation()
                    .to_scale_rotation_translation();
                let mut rotation = glam::Vec3::from(rotation.to_euler(glam::EulerRot::XYZ));
                Self::transformation_detail(&mut scale, &mut rotation, &mut translation, ui);
                let rotation =
                    glam::Quat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
                *component.get_interactive_transformation() =
                    glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);
            }
        }

        event
    }

    pub fn transformation_detail(
        scale: &mut glam::Vec3,
        rotation: &mut glam::Vec3,
        translation: &mut glam::Vec3,
        ui: &mut egui::Ui,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("location ");
                ui.add(
                    egui::DragValue::new(&mut translation.x)
                        .speed(0.1)
                        .prefix("x: "),
                );
                ui.add(
                    egui::DragValue::new(&mut translation.y)
                        .speed(0.1)
                        .prefix("y: "),
                );
                ui.add(
                    egui::DragValue::new(&mut translation.z)
                        .speed(0.1)
                        .prefix("z: "),
                );
            });
            ui.horizontal(|ui| {
                ui.label("scale ");
                ui.add(egui::DragValue::new(&mut scale.x).speed(0.1).prefix("x: "));
                ui.add(egui::DragValue::new(&mut scale.y).speed(0.1).prefix("y: "));
                ui.add(egui::DragValue::new(&mut scale.z).speed(0.1).prefix("z: "));
            });
            Self::rotation_detail(rotation, ui);
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

    fn rotation_detail(rotation: &mut glam::Vec3, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("rotation ");
            ui.add(
                egui::DragValue::new(&mut rotation.x)
                    .speed(0.1)
                    .prefix("x: "),
            );
            ui.add(
                egui::DragValue::new(&mut rotation.y)
                    .speed(0.1)
                    .prefix("y: "),
            );
            ui.add(
                egui::DragValue::new(&mut rotation.z)
                    .speed(0.1)
                    .prefix("z: "),
            );
        });
    }
}
