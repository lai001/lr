use rs_engine::{
    actor::Actor, directional_light::DirectionalLight, scene_node::*,
    skeleton_mesh_component::SkeletonMeshComponent, static_mesh_component::StaticMeshComponent,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};

pub struct UpdateMaterial {
    pub selected_object: ESelectedObjectType,
    pub old: Option<url::Url>,
    pub new: Option<url::Url>,
}

pub enum EEventType {
    UpdateMaterial(UpdateMaterial),
    UpdateDirectionalLight(
        SingleThreadMutType<DirectionalLight>,
        f32,
        f32,
        f32,
        f32,
        f32,
    ),
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
                ui.label(format!("Type: Actor"));
                ui.label(actor.borrow().name.clone());
            }
            ESelectedObjectType::SceneComponent(scene_component) => {
                ui.label(format!("Type: SceneComponent"));

                let mut component = scene_component.borrow_mut();
                ui.label(component.name.clone());

                Self::transformation_detail_mut(component.get_transformation_mut(), ui);
                Self::transformation_detail(&component.get_final_transformation(), ui);
            }
            ESelectedObjectType::StaticMeshComponent(static_mesh_component) => {
                ui.label(format!("Type: StaticMeshComponent"));

                let mut component = static_mesh_component.borrow_mut();
                ui.label(component.name.clone());

                Self::transformation_detail_mut(component.get_transformation_mut(), ui);
                Self::transformation_detail(&component.get_final_transformation(), ui);

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
                ui.label(format!("Type: SkeletonMeshComponent"));

                let mut component = skeleton_mesh_component.borrow_mut();
                ui.label(component.name.clone());

                Self::transformation_detail_mut(component.get_transformation_mut(), ui);
                Self::transformation_detail(&component.get_final_transformation(), ui);

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
                ui.label(format!("Type: DirectionalLight"));
                let directional_light_clone = directional_light.clone();
                let mut component = directional_light.borrow_mut();
                Self::transformation_detail_mut(component.get_transformation_mut(), ui);

                let mut is_changed = false;

                let mut left = component.left;
                is_changed = is_changed
                    || ui
                        .add(egui::DragValue::new(&mut left).speed(0.1).prefix("Left: "))
                        .changed();

                let mut right = component.right;
                is_changed = is_changed
                    || ui
                        .add(
                            egui::DragValue::new(&mut right)
                                .speed(0.1)
                                .prefix("Right: "),
                        )
                        .changed();

                let mut top = component.top;
                is_changed = is_changed
                    || ui
                        .add(egui::DragValue::new(&mut top).speed(0.1).prefix("Top: "))
                        .changed();

                let mut bottom = component.bottom;
                is_changed = is_changed
                    || ui
                        .add(
                            egui::DragValue::new(&mut bottom)
                                .speed(0.1)
                                .prefix("Bottom: "),
                        )
                        .changed();

                let mut far = component.far;
                is_changed = is_changed
                    || ui
                        .add(egui::DragValue::new(&mut far).speed(0.1).prefix("Far: "))
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

    pub fn transformation_detail(transformation: &glam::Mat4, ui: &mut egui::Ui) {
        let (scale, rotation, translation) = transformation.to_scale_rotation_translation();
        let rotation = glam::Vec3::from(rotation.to_euler(glam::EulerRot::XYZ));
        Self::affine_detail(&scale, &rotation, &translation, ui);
    }

    pub fn transformation_detail_mut(transformation: &mut glam::Mat4, ui: &mut egui::Ui) {
        let (mut scale, rotation, mut translation) = transformation.to_scale_rotation_translation();
        let mut rotation = glam::Vec3::from(rotation.to_euler(glam::EulerRot::XYZ));
        Self::affine_detail_mut(&mut scale, &mut rotation, &mut translation, ui);
        let rotation =
            glam::Quat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
        *transformation = glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);
    }

    pub fn affine_detail_mut(
        scale: &mut glam::Vec3,
        rotation: &mut glam::Vec3,
        translation: &mut glam::Vec3,
        ui: &mut egui::Ui,
    ) {
        ui.vertical(|ui| {
            Self::detail_view_mut(translation, ui, "Location");
            Self::detail_view_mut(scale, ui, "Scale");
            Self::detail_view_mut(rotation, ui, "Rotation");
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

    pub fn affine_detail(
        scale: &glam::Vec3,
        rotation: &glam::Vec3,
        translation: &glam::Vec3,
        ui: &mut egui::Ui,
    ) {
        ui.vertical(|ui| {
            Self::detail_view(translation, ui, "Location");
            Self::detail_view(scale, ui, "Scale");
            Self::detail_view(rotation, ui, "Rotation");
        });
    }

    fn detail_view(value: &glam::Vec3, ui: &mut egui::Ui, label: &str) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "{} x: {}, y: {}, z: {}",
                label, value.x, value.y, value.z
            ));
        });
    }

    fn detail_view_mut(value: &mut glam::Vec3, ui: &mut egui::Ui, label: &str) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(egui::DragValue::new(&mut value.x).speed(0.1).prefix("x: "));
            ui.add(egui::DragValue::new(&mut value.y).speed(0.1).prefix("y: "));
            ui.add(egui::DragValue::new(&mut value.z).speed(0.1).prefix("z: "));
        });
    }
}
