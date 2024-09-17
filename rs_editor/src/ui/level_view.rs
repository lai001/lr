use egui::{Context, ScrollArea, Ui};
use rs_engine::{actor::Actor, directional_light::DirectionalLight, scene_node::SceneNode};
use rs_foundation::new::SingleThreadMutType;
use std::{cell::RefCell, rc::Rc};

pub enum EClickEventType {
    Actor(SingleThreadMutType<Actor>),
    SceneNode(SingleThreadMutType<SceneNode>),
    CreateDirectionalLight,
    DirectionalLight(SingleThreadMutType<DirectionalLight>),
}

fn draw_scene_node(
    ui: &mut Ui,
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
        }
    };
    let id = ui.make_persistent_id(name.clone());
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            if ui.button(name).clicked() {
                *event = Some(EClickEventType::SceneNode(scene_node.clone()));
            }
        })
        .body(|ui| {
            for child in &scene_node.borrow().childs {
                draw_scene_node(ui, child.clone(), event);
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
            if ui.button(name).clicked() {
                *event = Some(EClickEventType::Actor(actor.clone()));
            }
        })
        .body(|ui| {
            draw_scene_node(ui, actor.borrow().scene_node.clone(), event);
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
                    if ui.button(format!("DirectionalLight_{}", index)).clicked() {
                        event = Some(EClickEventType::DirectionalLight(light.clone()));
                    }
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
