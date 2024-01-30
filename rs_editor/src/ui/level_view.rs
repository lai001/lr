use egui::{Context, ScrollArea, Ui, Window};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub enum EClickEventType {
    Node(Rc<RefCell<crate::level::Node>>),
}

fn level_node(
    ui: &mut Ui,
    node: Rc<RefCell<crate::level::Node>>,
    event: &mut Option<EClickEventType>,
) {
    let _node = node.as_ref().borrow();
    let id = _node.id;
    let name = &_node.name;
    let childs = &_node.childs;
    let id = ui.make_persistent_id(id);
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| {
            if ui.button(name.clone()).clicked() {
                *event = Some(EClickEventType::Node(node.clone()));
            }
        })
        .body(|ui| {
            for child_node in childs {
                level_node(ui, child_node.clone(), event);
            }
        });
}

pub fn draw(
    context: &Context,
    is_open: &mut bool,
    level: &crate::level::Level,
) -> Option<EClickEventType> {
    let mut event: Option<EClickEventType> = None;
    Window::new(format!("Level({})", level.name))
        .open(is_open)
        .show(context, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for node in &level.nodes {
                    level_node(ui, node.clone(), &mut event);
                }
            });
        });
    event
}
