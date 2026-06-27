use super::misc::{render_combo_box, render_combo_box_not_null};
use rapier3d::prelude::{Ball, Cuboid, HalfSpace, RigidBodyType};
use rs_engine::{
    actor::Actor,
    components::component::Component,
    directional_light::DirectionalLight,
    physics_ability::{EShapeType, MeshOptions},
    scene_node::*,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_localization::t;

pub struct UpdateMaterial {
    pub selected_object: ESelectedObjectType,
    pub old: Option<url::Url>,
    pub new: Option<url::Url>,
}

pub struct UpdateAnimation {
    pub selected_object: ESelectedObjectType,
    pub old: Option<url::Url>,
    pub new: Option<url::Url>,
}

pub struct UpdateStaticMesh {
    pub selected_object: ESelectedObjectType,
    pub old: Option<url::Url>,
    pub new: Option<url::Url>,
}

pub struct UpdatePhysicsShapeType {
    pub selected_object: ESelectedObjectType,
    pub shape_type: EShapeType,
}

pub enum EEventType {
    UpdateMaterial(UpdateMaterial),
    UpdateAnimation(UpdateAnimation),
    UpdateStaticMesh(UpdateStaticMesh),
    UpdateDirectionalLight(
        SingleThreadMutType<DirectionalLight>,
        f32,
        f32,
        f32,
        f32,
        f32,
    ),
    ChangeName(ESelectedObjectType, String),
    UpdateIsEnableMultiresolution(ESelectedObjectType, bool, bool),
    UpdatePhysicsShapeType(UpdatePhysicsShapeType),
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

    fn edit_name(name: &str, ui: &mut egui::Ui) -> Option<String> {
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

    pub fn draw(&mut self, ui: &mut egui::Ui) -> Option<EEventType> {
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
                let mut scene_node = scene_node.borrow_mut();
                match &mut scene_node.component {
                    EComponentType::SceneComponent(scene_component) => {
                        ui.label(format!("{}", t!("Type: SceneComponent")));

                        let mut component = scene_component.borrow_mut();
                        if let Some(new_name) = Self::edit_name(&component.name, ui) {
                            event = Some(EEventType::ChangeName(selected_object_clone, new_name));
                        }

                        Self::transformation_widget_mut(component.get_transformation_mut(), ui);
                        Self::transformation_widget(&component.get_final_transformation(), ui);
                    }
                    EComponentType::StaticMeshComponent(static_mesh_component) => {
                        ui.label(format!("{}", t!("Type: StaticMeshComponent")));

                        let mut component = static_mesh_component.borrow_mut();
                        if let Some(new_name) = Self::edit_name(&component.name, ui) {
                            event = Some(EEventType::ChangeName(
                                selected_object_clone.clone(),
                                new_name,
                            ));
                        }

                        Self::transformation_widget_mut(component.get_transformation_mut(), ui);
                        Self::transformation_widget(&component.get_final_transformation(), ui);

                        {
                            let mut current_url = component.material_url.as_ref();
                            let candidate_items = self.materials.borrow();
                            let old_url = current_url.cloned();
                            let is_changed = render_combo_box(
                                ui,
                                t!("Material"),
                                Some(egui::Id::new("Material")),
                                &mut current_url,
                                &candidate_items,
                            );
                            if is_changed {
                                event = Some(EEventType::UpdateMaterial(UpdateMaterial {
                                    selected_object: selected_object_clone.clone(),
                                    old: old_url,
                                    new: current_url.cloned(),
                                }));
                            }
                        }

                        {
                            let mut current_url = component.static_mesh.as_ref();
                            let candidate_items = self.static_meshes.borrow();
                            let old_url = current_url.cloned();
                            let is_changed = render_combo_box(
                                ui,
                                t!("Static mesh"),
                                Some(egui::Id::new("StaticMesh")),
                                &mut current_url,
                                &candidate_items,
                            );
                            if is_changed {
                                event = Some(EEventType::UpdateStaticMesh(UpdateStaticMesh {
                                    selected_object: selected_object_clone.clone(),
                                    old: old_url,
                                    new: current_url.cloned(),
                                }));
                            }
                        }

                        let body_types = vec![
                            RigidBodyType::Dynamic,
                            RigidBodyType::Fixed,
                            RigidBodyType::KinematicPositionBased,
                            RigidBodyType::KinematicVelocityBased,
                        ];
                        let _ = render_combo_box_not_null(
                            ui,
                            t!("Rigid body type"),
                            "Rigid body type",
                            &mut component.physics.rigid_body_type,
                            body_types,
                        );

                        let candidate_items: Vec<String> = vec![
                            t!("HalfSpace").to_string(),
                            t!("Ball").to_string(),
                            t!("Cuboid").to_string(),
                            t!("Mesh").to_string(),
                        ];
                        let mut current_value: String;
                        match &component.physics.shape_type {
                            rs_engine::physics_ability::EShapeType::HalfSpace(_) => {
                                current_value = candidate_items[0].clone();
                            }
                            rs_engine::physics_ability::EShapeType::Ball(_) => {
                                current_value = candidate_items[1].clone();
                            }
                            rs_engine::physics_ability::EShapeType::Cuboid(_) => {
                                current_value = candidate_items[2].clone();
                            }
                            rs_engine::physics_ability::EShapeType::Mesh(_) => {
                                current_value = candidate_items[3].clone();
                            }
                        }
                        let mut is_changed = render_combo_box_not_null(
                            ui,
                            t!("Shape type"),
                            "Shape type",
                            &mut current_value,
                            candidate_items,
                        );
                        if is_changed {
                            if current_value == t!("HalfSpace") {
                                component.physics.shape_type =
                                    EShapeType::HalfSpace(HalfSpace::new(glam::Vec3::Z))
                            } else if current_value == t!("Ball") {
                                component.physics.shape_type = EShapeType::Ball(Ball::new(5.0))
                            } else if current_value == t!("Cuboid") {
                                component.physics.shape_type =
                                    EShapeType::Cuboid(Cuboid::new(glam::Vec3::splat(5.0)))
                            } else if current_value == t!("Mesh") {
                                component.physics.shape_type = EShapeType::Mesh(MeshOptions {
                                    mesh_url: None,
                                    is_use_convex_decomposition: false,
                                })
                            }
                        }

                        match &mut component.physics.shape_type {
                            rs_engine::physics_ability::EShapeType::HalfSpace(half_space) => {
                                is_changed = Self::vec3_widget_mut(
                                    &mut half_space.normal,
                                    ui,
                                    t!("HalfSpace"),
                                    false,
                                );
                            }
                            rs_engine::physics_ability::EShapeType::Ball(ball) => {
                                is_changed =
                                    Self::vec1_widget_mut(&mut ball.radius, ui, t!("Ball"));
                            }
                            rs_engine::physics_ability::EShapeType::Cuboid(cuboid) => {
                                is_changed = Self::vec3_widget_mut(
                                    &mut cuboid.half_extents,
                                    ui,
                                    t!("Cuboid"),
                                    false,
                                );
                            }
                            rs_engine::physics_ability::EShapeType::Mesh(mesh_options) => {
                                let mut current_url = mesh_options.mesh_url.as_ref();
                                let candidate_items = self.static_meshes.borrow();
                                is_changed = render_combo_box(
                                    ui,
                                    t!("Static mesh"),
                                    Some(egui::Id::new("ShapeTypeStaticMesh")),
                                    &mut current_url,
                                    &candidate_items,
                                );
                                mesh_options.mesh_url = current_url.cloned();
                            }
                        }
                        if is_changed {
                            event =
                                Some(EEventType::UpdatePhysicsShapeType(UpdatePhysicsShapeType {
                                    selected_object: selected_object_clone.clone(),
                                    shape_type: component.physics.shape_type.clone(),
                                }));
                        }

                        if ui
                            .checkbox(
                                &mut component.is_enable_multiresolution,
                                t!("Is enable multiresolution").as_ref(),
                            )
                            .changed()
                        {
                            event = Some(EEventType::UpdateIsEnableMultiresolution(
                                selected_object_clone.clone(),
                                !component.is_enable_multiresolution,
                                component.is_enable_multiresolution,
                            ));
                        }
                    }
                    EComponentType::SkeletonMeshComponent(skeleton_mesh_component) => {
                        ui.label(format!("{}", t!("Type: SkeletonMeshComponent")));

                        let mut component = skeleton_mesh_component.borrow_mut();
                        if let Some(new_name) = Self::edit_name(&component.name, ui) {
                            event = Some(EEventType::ChangeName(
                                selected_object_clone.clone(),
                                new_name,
                            ));
                        }

                        Self::transformation_widget_mut(component.get_transformation_mut(), ui);
                        Self::transformation_widget(&component.get_final_transformation(), ui);

                        egui::ComboBox::from_label(t!("Animation").as_ref())
                            .selected_text(format!("{}", {
                                match &component.animation_url {
                                    Some(animation_url) => animation_url.to_string(),
                                    None => t!("None").to_string(),
                                }
                            }))
                            .show_ui(ui, |ui| {
                                let mut collection: Vec<Option<url::Url>> = vec![];
                                collection.push(None);
                                collection.append(
                                    &mut self
                                        .animations
                                        .borrow()
                                        .iter()
                                        .map(|x| Some(x.clone()))
                                        .collect(),
                                );

                                for animation in collection {
                                    let old = component.animation_url.clone();
                                    let text = animation
                                        .as_ref()
                                        .map(|x| x.to_string())
                                        .unwrap_or(t!("None").to_string());
                                    let is_changed = ui
                                        .selectable_value(
                                            &mut component.animation_url,
                                            animation.clone(),
                                            text,
                                        )
                                        .changed();
                                    if is_changed {
                                        event =
                                            Some(EEventType::UpdateAnimation(UpdateAnimation {
                                                selected_object: selected_object_clone.clone(),
                                                old,
                                                new: animation.clone(),
                                            }));
                                    }
                                }
                            });

                        egui::ComboBox::from_label(t!("Material").as_ref())
                            .selected_text(format!("{}", {
                                match &component.material_url {
                                    Some(material_url) => material_url.to_string(),
                                    None => t!("None").to_string(),
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
                                        .unwrap_or(t!("None").to_string());
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
                    EComponentType::CameraComponent(component) => {
                        ui.label(format!("{}", t!("Type: CameraComponent")));
                        let mut component = component.borrow_mut();
                        if let Some(new_name) = Self::edit_name(&component.name, ui) {
                            event = Some(EEventType::ChangeName(
                                selected_object_clone.clone(),
                                new_name,
                            ));
                        }
                        ui.checkbox(
                            &mut component.is_show_preview,
                            t!("Is show frustum").as_ref(),
                        );

                        Self::transformation_widget_mut(component.get_transformation_mut(), ui);
                        Self::transformation_widget(&component.get_final_transformation(), ui);
                        ui.checkbox(&mut component.is_enable, t!("Is enable").as_ref());
                    }
                    EComponentType::CollisionComponent(component) => {
                        ui.label(format!("{}", t!("Type: CollisionComponent")));
                        let mut component = component.borrow_mut();
                        if let Some(new_name) = Self::edit_name(&component.name, ui) {
                            event = Some(EEventType::ChangeName(
                                selected_object_clone.clone(),
                                new_name,
                            ));
                        }
                        ui.checkbox(
                            &mut component.is_show_preview,
                            t!("Is show preview").as_ref(),
                        );

                        Self::transformation_widget_mut(component.get_transformation_mut(), ui);
                        Self::transformation_widget(&component.get_final_transformation(), ui);
                    }
                    EComponentType::SpotLightComponent(component) => {
                        ui.label(format!("{}", t!("Type: SpotLightComponent")));
                        let mut component = component.borrow_mut();
                        if let Some(new_name) = Self::edit_name(&component.name, ui) {
                            event = Some(EEventType::ChangeName(
                                selected_object_clone.clone(),
                                new_name,
                            ));
                        }
                        let mut transformation = component.get_transformation();
                        Self::transformation_widget_mut(&mut transformation, ui);
                        component.set_transformation(transformation);
                        Self::transformation_widget(&component.get_final_transformation(), ui);

                        ui.vertical(|ui| {
                            Self::vec3_widget_mut(
                                &mut component.spot_light.light.ambient,
                                ui,
                                t!("Ambient"),
                                true,
                            );
                            Self::vec3_widget_mut(
                                &mut component.spot_light.light.diffuse,
                                ui,
                                t!("Diffuse"),
                                true,
                            );
                            Self::vec3_widget_mut(
                                &mut component.spot_light.light.specular,
                                ui,
                                t!("Specular"),
                                true,
                            );
                            ui.add(
                                egui::DragValue::new(&mut component.spot_light.light.constant)
                                    .speed(0.1)
                                    .prefix(t!("Constant: ").as_ref()),
                            );
                            ui.add(
                                egui::DragValue::new(&mut component.spot_light.light.linear)
                                    .speed(0.1)
                                    .prefix(t!("Linear: ").as_ref()),
                            );
                            ui.add(
                                egui::DragValue::new(&mut component.spot_light.light.quadratic)
                                    .speed(0.1)
                                    .prefix(t!("Quadratic: ").as_ref()),
                            );
                            ui.add(
                                egui::DragValue::new(&mut component.spot_light.cut_off)
                                    .speed(0.1)
                                    .prefix(t!("Cut off: ").as_ref()),
                            );
                            ui.add(
                                egui::DragValue::new(&mut component.spot_light.outer_cut_off)
                                    .speed(0.1)
                                    .prefix(t!("Outer cut off: ").as_ref()),
                            );
                        });
                    }
                    EComponentType::PointLightComponent(component) => {
                        ui.label(format!("{}", t!("Type: PointLightComponent")));
                        let mut component = component.borrow_mut();
                        if let Some(new_name) = Self::edit_name(&component.name, ui) {
                            event = Some(EEventType::ChangeName(
                                selected_object_clone.clone(),
                                new_name,
                            ));
                        }
                        let mut transformation = component.get_transformation();
                        Self::transformation_widget_mut(&mut transformation, ui);
                        component.set_transformation(transformation);
                        Self::transformation_widget(&component.get_final_transformation(), ui);

                        ui.vertical(|ui| {
                            Self::vec3_widget_mut(
                                &mut component.point_light.ambient,
                                ui,
                                t!("Ambient"),
                                true,
                            );
                            Self::vec3_widget_mut(
                                &mut component.point_light.diffuse,
                                ui,
                                t!("Diffuse"),
                                true,
                            );
                            Self::vec3_widget_mut(
                                &mut component.point_light.specular,
                                ui,
                                t!("Specular"),
                                true,
                            );
                            ui.add(
                                egui::DragValue::new(&mut component.point_light.constant)
                                    .speed(0.1)
                                    .prefix(t!("Constant: ").as_ref()),
                            );
                            ui.add(
                                egui::DragValue::new(&mut component.point_light.linear)
                                    .speed(0.1)
                                    .prefix(t!("Linear: ").as_ref()),
                            );
                            ui.add(
                                egui::DragValue::new(&mut component.point_light.quadratic)
                                    .speed(0.1)
                                    .prefix(t!("Quadratic: ").as_ref()),
                            );
                        });
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

    fn vec3_widget_mut(
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

    fn vec1_widget_mut<Num: egui::emath::Numeric>(
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
