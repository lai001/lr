use crate::ui::{
    UIEvent,
    component_edit::{ComponentEditable, UIComponentPropertyEvent},
    object_property_view::ObjectPropertyView,
};
use egui::Ui;
use rs_content_manager::content_manager::ContentManager;
use rs_engine::{
    components::component::Component, engine::Engine,
    skeleton_mesh_component::SkeletonMeshComponent,
};
use rust_i18n::t;
use std::borrow::Cow;

struct UpdateMaterial {
    new: Option<url::Url>,
}

struct UpdateAnimation {
    new: Option<url::Url>,
}

enum EEventType {
    UpdateMaterial(UpdateMaterial),
    UpdateAnimation(UpdateAnimation),
}

impl UIEvent for EEventType {}
impl UIComponentPropertyEvent for EEventType {}

pub struct SkeletonMeshComponentEdit {}

impl ComponentEditable for SkeletonMeshComponentEdit {
    fn edit(
        &mut self,
        ui: &mut Ui,
        component: &mut dyn Component,
        engine: &mut Engine,
        content_manager: &mut ContentManager,
        object_property_view: &ObjectPropertyView,
    ) -> Option<Box<dyn super::UIComponentPropertyEvent>> {
        let _ = content_manager;
        let _ = engine;

        let component = component
            .downcast_mut::<SkeletonMeshComponent>()
            .expect("Matched type");

        let mut event: Option<EEventType> = None;

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
                    &mut object_property_view
                        .animations
                        .borrow()
                        .iter()
                        .map(|x| Some(x.clone()))
                        .collect(),
                );

                for animation in collection {
                    let text = animation
                        .as_ref()
                        .map(|x| x.to_string())
                        .unwrap_or(t!("None").to_string());
                    let is_changed = ui
                        .selectable_value(&mut component.animation_url, animation.clone(), text)
                        .changed();
                    if is_changed {
                        event = Some(EEventType::UpdateAnimation(UpdateAnimation {
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
                    &mut object_property_view
                        .materials
                        .borrow()
                        .iter()
                        .map(|x| Some(x.clone()))
                        .collect(),
                );

                for material in collection {
                    let text = material
                        .as_ref()
                        .map(|x| x.to_string())
                        .unwrap_or(t!("None").to_string());
                    let is_changed = ui
                        .selectable_value(&mut component.material_url, material.clone(), text)
                        .changed();
                    if is_changed {
                        event = Some(EEventType::UpdateMaterial(UpdateMaterial {
                            new: material.clone(),
                        }));
                    }
                }
            });

        event.map(|x| Box::new(x) as Box<dyn super::UIComponentPropertyEvent>)
    }

    fn on_process_event(
        &self,
        editor_context: &mut crate::editor_context::EditorContext,
        component: &mut dyn Component,
        event: Box<dyn UIComponentPropertyEvent>,
    ) {
        let Ok(event) = event.downcast::<EEventType>() else {
            return;
        };
        let crate::editor_context::EditObjectContext {
            player_viewport,
            engine,
            project_context,
            ..
        } = editor_context.edit_object_context();
        let content_manager = project_context.content_manager.clone();
        let content_manager = content_manager.borrow();
        let skeleton_mesh_component = component
            .downcast_mut::<SkeletonMeshComponent>()
            .expect("Matched type");

        match *event {
            EEventType::UpdateMaterial(UpdateMaterial { new }) => {
                if let Some(url) = new {
                    let files = content_manager.content_files();
                    skeleton_mesh_component.set_material(engine, url, &files, player_viewport);
                }
            }
            EEventType::UpdateAnimation(UpdateAnimation { new }) => {
                let files = content_manager.content_map();
                skeleton_mesh_component.set_animation(
                    new,
                    engine.get_resource_manager().clone(),
                    &files,
                );
            }
        }
    }

    fn display_type_name(&self) -> Cow<'static, str> {
        t!("Type: SkeletonMeshComponent")
    }
}
