use crate::ui::{
    UIEvent,
    component_edit::{ComponentEditable, UIComponentPropertyEvent},
    misc::{render_combo_box, render_combo_box_not_null, render_data_field},
    object_property_view::ObjectPropertyView,
};
use rapier3d::{
    dynamics::RigidBodyType,
    geometry::{Ball, Cuboid, HalfSpace},
};
use rs_engine::{
    components::component::Component,
    physics_ability::{EShapeType, MeshOptions},
    static_mesh_component::StaticMeshComponent,
};
use rust_i18n::t;

struct UpdateMaterial {
    new: Option<url::Url>,
}

struct UpdateStaticMesh {
    new: Option<url::Url>,
}

enum EEventType {
    UpdateMaterial(UpdateMaterial),
    UpdateStaticMesh(UpdateStaticMesh),
    UpdatePhysicsShapeType,
}

impl UIEvent for EEventType {}
impl UIComponentPropertyEvent for EEventType {}

pub struct StaticMeshComponentEdit {}

impl StaticMeshComponentEdit {}

impl ComponentEditable for StaticMeshComponentEdit {
    fn display_type_name(&self) -> std::borrow::Cow<'static, str> {
        t!("Type: StaticMeshComponent")
    }

    fn edit(
        &mut self,
        ui: &mut egui::Ui,
        component: &mut dyn rs_engine::components::component::Component,
        engine: &mut rs_engine::engine::Engine,
        content_manager: &mut rs_content_manager::content_manager::ContentManager,
        object_property_view: &ObjectPropertyView,
    ) -> Option<Box<dyn super::UIComponentPropertyEvent>> {
        let _ = content_manager;
        let _ = engine;
        let component = component
            .downcast_mut::<StaticMeshComponent>()
            .expect("Matched type");
        let mut event: Option<EEventType> = None;

        {
            let mut current_url = component.material_url.as_ref();
            let candidate_items = object_property_view.materials.borrow();
            let is_changed = render_combo_box(
                ui,
                t!("Material"),
                Some(egui::Id::new("Material")),
                &mut current_url,
                &candidate_items,
            );
            if is_changed {
                event = Some(EEventType::UpdateMaterial(UpdateMaterial {
                    new: current_url.cloned(),
                }));
            }
        }

        {
            let mut current_url = component.static_mesh.as_ref();
            let candidate_items = object_property_view.static_meshes.borrow();
            let is_changed = render_combo_box(
                ui,
                t!("Static mesh"),
                Some(egui::Id::new("StaticMesh")),
                &mut current_url,
                &candidate_items,
            );
            if is_changed {
                event = Some(EEventType::UpdateStaticMesh(UpdateStaticMesh {
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
                component.physics.shape_type = EShapeType::HalfSpace(HalfSpace::new(glam::Vec3::Z))
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
                is_changed = ObjectPropertyView::vec3_widget_mut(
                    &mut half_space.normal,
                    ui,
                    t!("HalfSpace"),
                    false,
                );
            }
            rs_engine::physics_ability::EShapeType::Ball(ball) => {
                is_changed = ObjectPropertyView::vec1_widget_mut(&mut ball.radius, ui, t!("Ball"));
            }
            rs_engine::physics_ability::EShapeType::Cuboid(cuboid) => {
                is_changed = ObjectPropertyView::vec3_widget_mut(
                    &mut cuboid.half_extents,
                    ui,
                    t!("Cuboid"),
                    false,
                );
            }
            rs_engine::physics_ability::EShapeType::Mesh(mesh_options) => {
                let mut current_url = mesh_options.mesh_url.as_ref();
                let candidate_items = object_property_view.static_meshes.borrow();
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
            event = Some(EEventType::UpdatePhysicsShapeType);
        }

        ui.checkbox(
            &mut component.is_enable_multiresolution,
            t!("Is enable multiresolution").as_ref(),
        );

        egui::CollapsingHeader::new(t!("Material Paramenters"))
            .default_open(true)
            .show(ui, |ui| {
                component.update_uniform_map(|uniform_map| {
                    let mut is_need_update = false;
                    uniform_map.update(|name, data_value| {
                        ui.horizontal(|ui| {
                            ui.label(name);
                            is_need_update |= render_data_field(ui, data_value);
                        });
                    });
                    is_need_update
                });
            });

        event.map(|x| Box::new(x) as Box<dyn super::UIComponentPropertyEvent>)
    }

    fn on_process_event(
        &self,
        editor_context: &mut crate::editor_context::EditorContext,
        component: &mut dyn rs_engine::components::component::Component,
        event: Box<dyn UIComponentPropertyEvent>,
    ) {
        let Ok(event) = event.downcast::<EEventType>() else {
            return;
        };

        let static_mesh_component = component
            .downcast_mut::<StaticMeshComponent>()
            .expect("Matched type");

        let crate::editor_context::EditObjectContext {
            player_viewport,
            engine,
            project_context,
            data_source,
            ..
        } = editor_context.edit_object_context();

        let content_manager = project_context.content_manager.clone();
        let content_manager = content_manager.borrow();

        match *event {
            EEventType::UpdateMaterial(UpdateMaterial { new, .. }) => {
                let content_files = content_manager.content_files();
                static_mesh_component.set_material(
                    engine,
                    new.clone(),
                    content_files,
                    player_viewport,
                );
            }
            EEventType::UpdateStaticMesh(UpdateStaticMesh { new, .. }) => {
                let Some(active_level) = data_source.level.as_mut() else {
                    return;
                };
                let files = content_manager.content_map();
                let static_mesh_url = new;
                static_mesh_component.set_static_mesh_url(
                    static_mesh_url,
                    engine.get_resource_manager().clone(),
                    engine,
                    &files,
                    player_viewport,
                );
                let mut active_level = active_level.borrow_mut();
                let physics = active_level.get_physics_mut();
                if let Some(physics) = physics {
                    static_mesh_component.initialize_physics(engine, physics, &files);
                }
            }
            EEventType::UpdatePhysicsShapeType => {
                let Some(active_level) = data_source.level.as_mut() else {
                    return;
                };
                let mut active_level = active_level.borrow_mut();
                let physics = active_level.get_physics_mut();
                let Some(level_physics) = physics else { return };
                let files = content_manager.content_map();
                static_mesh_component.initialize_physics(engine, level_physics, &files);
            }
        }
    }
}
