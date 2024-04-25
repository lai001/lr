use egui::{Context, ScrollArea, Ui};
use std::{cell::RefCell, rc::Rc};

pub enum EClickEventType {
    Actor(Rc<RefCell<rs_engine::actor::Actor>>),
}

fn level_node(
    ui: &mut Ui,
    actor: Rc<RefCell<rs_engine::actor::Actor>>,
    event: &mut Option<EClickEventType>,
) {
    let _actor = actor.as_ref().borrow();
    let name = &_actor.name;
    let id = ui.make_persistent_id(name);
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| {
            ui.label(name);
        })
        .body(|ui| {});
}

pub fn draw(
    window: egui::Window,
    context: &Context,
    is_open: &mut bool,
    level: &rs_engine::content::level::Level,
) -> Option<EClickEventType> {
    let mut event: Option<EClickEventType> = None;
    window.open(is_open).show(context, |ui| {
        ui.label(format!("name: {}", level.get_name()));
        ScrollArea::vertical().show(ui, |ui| {
            for actor in &level.actors {
                level_node(ui, actor.clone(), &mut event);
            }
        });
    });
    event
}
