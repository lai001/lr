use crate::component_factory::ComponentFactory;
use egui::{Context, ScrollArea, Ui};
use rs_engine::{actor::Actor, directional_light::DirectionalLight, scene_node::SceneNode};
use rs_foundation::new::SingleThreadMutType;
use rs_localization::t;
use std::{cell::RefCell, rc::Rc};

pub enum EClickEventType {
    SingleClickActor(SingleThreadMutType<Actor>),
    CreateActor,
    CreateCameraHere,
    DeleteActor(SingleThreadMutType<Actor>),
    DuplicateActor(SingleThreadMutType<Actor>),
    SingleClickSceneNode(SingleThreadMutType<SceneNode>),
    CreateDirectionalLight,
    DirectionalLight(SingleThreadMutType<DirectionalLight>),
    DeleteDirectionalLight(SingleThreadMutType<DirectionalLight>),
    CreateCameraComponent(SingleThreadMutType<SceneNode>),
    CreateSceneComponent(SingleThreadMutType<SceneNode>),
    CreateStaticMeshComponent(SingleThreadMutType<SceneNode>),
    CopyPath(SingleThreadMutType<Actor>, SingleThreadMutType<SceneNode>),
    DeleteNode(SingleThreadMutType<Actor>, SingleThreadMutType<SceneNode>),
    CreateCollisionComponent(SingleThreadMutType<Actor>, SingleThreadMutType<SceneNode>),
    CreateSpotLightComponent(SingleThreadMutType<SceneNode>),
    CreatePointLightComponent(SingleThreadMutType<SceneNode>),
    CreateComponent(String, SingleThreadMutType<SceneNode>),
}

fn draw_scene_node(
    ui: &mut Ui,
    actor: SingleThreadMutType<rs_engine::actor::Actor>,
    scene_node: SingleThreadMutType<SceneNode>,
    event: &mut Option<EClickEventType>,
    component_factory: &ComponentFactory,
) {
    let name = { scene_node.borrow().component().get_name() };
    let id = ui.make_persistent_id(name.clone());
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            let response = ui.button(name);
            if response.clicked() {
                *event = Some(EClickEventType::SingleClickSceneNode(scene_node.clone()));
            } else {
                response.context_menu(|ui| {
                    ui.menu_button(t!("Add"), |ui| {
                        for (name, creator) in component_factory.creators() {
                            let display_name = creator.display_name();
                            let response = ui.button(display_name);
                            if response.clicked() {
                                *event = Some(EClickEventType::CreateComponent(
                                    name.clone(),
                                    scene_node.clone(),
                                ));
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        }
                    });
                    ui.menu_button(t!("Copy"), |ui| {
                        let response = ui.button(t!("Path"));
                        if response.clicked() {
                            *event =
                                Some(EClickEventType::CopyPath(actor.clone(), scene_node.clone()));
                            ui.close_kind(egui::UiKind::Menu);
                        }
                    });
                    let response = ui.button(t!("Delete"));
                    if response.clicked() {
                        *event = Some(EClickEventType::DeleteNode(
                            actor.clone(),
                            scene_node.clone(),
                        ));
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });
            }
        })
        .body(|ui| {
            for child in scene_node.borrow().childs() {
                draw_scene_node(ui, actor.clone(), child.clone(), event, component_factory);
            }
        });
}

fn level_node(
    ui: &mut Ui,
    actor: Rc<RefCell<rs_engine::actor::Actor>>,
    event: &mut Option<EClickEventType>,
    component_factory: &ComponentFactory,
) {
    let _actor = actor.as_ref().borrow();
    let name = &_actor.name;
    let id = ui.make_persistent_id(name);
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            let response = ui.button(name);
            if response.clicked() {
                *event = Some(EClickEventType::SingleClickActor(actor.clone()));
            } else {
                response.context_menu(|ui| {
                    let response = ui.button(t!("Duplicate"));
                    if response.clicked() {
                        *event = Some(EClickEventType::DuplicateActor(actor.clone()));
                        ui.close_kind(egui::UiKind::Menu);
                    }

                    let response = ui.button(t!("Delete"));
                    if response.clicked() {
                        *event = Some(EClickEventType::DeleteActor(actor.clone()));
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });
            }
        })
        .body(|ui| {
            draw_scene_node(
                ui,
                actor.clone(),
                actor.borrow().scene_node.clone(),
                event,
                component_factory,
            );
        });
}

pub fn draw(
    window: egui::Window,
    context: &Context,
    is_open: &mut bool,
    level: &rs_engine::content::level::Level,
    component_factory: &ComponentFactory,
) -> Option<EClickEventType> {
    let mut event: Option<EClickEventType> = None;
    window.open(is_open).show(context, |ui| {
        let response = ui.vertical(|ui| {
            ui.label(format!("{} {}", t!("Name: "), level.get_name()));
            ScrollArea::vertical().show(ui, |ui| {
                for (index, light) in level.directional_lights.iter().enumerate() {
                    let response = ui.button(t!("DirectionalLight [%{index}]", index = index));
                    if response.clicked() {
                        event = Some(EClickEventType::DirectionalLight(light.clone()));
                    }
                    response.context_menu(|ui| {
                        let response = ui.button(t!("Delete"));
                        if response.clicked() {
                            event = Some(EClickEventType::DeleteDirectionalLight(light.clone()));
                            ui.close_kind(egui::UiKind::Menu);
                        }
                    });
                }
                for actor in &level.actors {
                    level_node(ui, actor.clone(), &mut event, component_factory);
                }
            });
        });
        let interacted_response = response.response.interact(egui::Sense::all());
        interacted_response.context_menu(|ui| {
            ui.menu_button(t!("Add"), |ui| {
                if ui.button(t!("Directional Light")).clicked() {
                    event = Some(EClickEventType::CreateDirectionalLight);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button(t!("Actor")).clicked() {
                    event = Some(EClickEventType::CreateActor);
                    ui.close_kind(egui::UiKind::Menu);
                }
            });
            if ui.button(t!("Create camera here")).clicked() {
                event = Some(EClickEventType::CreateCameraHere);
                ui.close_kind(egui::UiKind::Menu);
            }
        });
    });
    event
}
