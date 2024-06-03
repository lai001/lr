use rs_engine::{actor::Actor, content::level::DirectionalLight, scene_node::*};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};

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

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let Some(selected_object) = self.selected_object.as_mut() else {
            return;
        };

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

                egui::ComboBox::from_label("material")
                    .selected_text(format!("{}", {
                        match &component.material_url {
                            Some(material_url) => material_url.to_string(),
                            None => "None".to_string(),
                        }
                    }))
                    .show_ui(ui, |ui| {
                        for material in self.materials.borrow_mut().clone() {
                            let text = material.to_string();
                            ui.selectable_value(&mut component.material_url, Some(material), text);
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
