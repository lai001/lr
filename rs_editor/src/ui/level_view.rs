use egui::{Context, ScrollArea, Ui};
use rs_engine::{actor::Actor, directional_light::DirectionalLight, scene_node::SceneNode};
use rs_foundation::new::SingleThreadMutType;
use std::{cell::RefCell, rc::Rc};

pub enum EClickEventType {
    Actor(SingleThreadMutType<Actor>),
    SceneNode(SingleThreadMutType<SceneNode>),
    CreateDirectionalLight,
    DirectionalLight(SingleThreadMutType<DirectionalLight>),
    DeleteDirectionalLight(SingleThreadMutType<DirectionalLight>),
    CreateCameraComponent(SingleThreadMutType<SceneNode>),
    DeleteNode(SingleThreadMutType<Actor>, SingleThreadMutType<SceneNode>),
}

fn draw_scene_node(
    ui: &mut Ui,
    actor: SingleThreadMutType<rs_engine::actor::Actor>,
    scene_node: SingleThreadMutType<SceneNode>,
    event: &mut Option<EClickEventType>,
) {
    let name = {
        match &scene_node.borrow().component {
            rs_engine::scene_node::EComponentType::SceneComponent(component) => {
                component.borrow().name.clone()
            }
            rs_engine::scene_node::EComponentType::StaticMeshComponent(component) => {
                component.borrow().name.clone()
            }
            rs_engine::scene_node::EComponentType::SkeletonMeshComponent(component) => {
                component.borrow().name.clone()
            }
            rs_engine::scene_node::EComponentType::CameraComponent(component) => {
                component.borrow().name.clone()
            }
        }
    };
    let id = ui.make_persistent_id(name.clone());
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            let response = ui.button(name);
            if response.clicked() {
                *event = Some(EClickEventType::SceneNode(scene_node.clone()));
            } else {
                response.context_menu(|ui| {
                    ui.menu_button("Add", |ui| {
                        let response = ui.button("Camera");
                        if response.clicked() {
                            *event =
                                Some(EClickEventType::CreateCameraComponent(scene_node.clone()));
                            ui.close_menu();
                        }
                    });
                    let response = ui.button("Delete");
                    if response.clicked() {
                        *event = Some(EClickEventType::DeleteNode(
                            actor.clone(),
                            scene_node.clone(),
                        ));
                        ui.close_menu();
                    }
                });
            }
        })
        .body(|ui| {
            for child in &scene_node.borrow().childs {
                draw_scene_node(ui, actor.clone(), child.clone(), event);
            }
        });
}

fn level_node(
    ui: &mut Ui,
    actor: Rc<RefCell<rs_engine::actor::Actor>>,
    event: &mut Option<EClickEventType>,
) {
    let _actor = actor.as_ref().borrow();
    let name = &_actor.name;
    let id = ui.make_persistent_id(name);
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            let response = ui.button(name);
            if response.clicked() {
                *event = Some(EClickEventType::Actor(actor.clone()));
            }
        })
        .body(|ui| {
            draw_scene_node(ui, actor.clone(), actor.borrow().scene_node.clone(), event);
        });
}

pub fn draw(
    window: egui::Window,
    context: &Context,
    is_open: &mut bool,
    level: &rs_engine::content::level::Level,
) -> Option<EClickEventType> {
    let mut event: Option<EClickEventType> = None;
    window.open(is_open).show(context, |ui| {
        let response = ui.vertical(|ui| {
            ui.label(format!("name: {}", level.get_name()));
            ScrollArea::vertical().show(ui, |ui| {
                for (index, light) in level.directional_lights.iter().enumerate() {
                    let response = ui.button(format!("DirectionalLight_{}", index));
                    if response.clicked() {
                        event = Some(EClickEventType::DirectionalLight(light.clone()));
                    }
                    response.context_menu(|ui| {
                        let response = ui.button("Delete");
                        if response.clicked() {
                            event = Some(EClickEventType::DeleteDirectionalLight(light.clone()));
                            ui.close_menu();
                        }
                    });
                }
                for actor in &level.actors {
                    level_node(ui, actor.clone(), &mut event);
                }
            });
        });
        response.response.context_menu(|ui| {
            if ui.button("Directional Light").clicked() {
                event = Some(EClickEventType::CreateDirectionalLight);
                ui.close_menu();
            }
        });
    });
    event
}
