use egui::{Context, ScrollArea, Ui, Window};

#[derive(Debug)]
pub enum EClickEventType {}

fn level_node(ui: &mut Ui, node: &crate::data_source::Node, event: &mut Option<EClickEventType>) {
    let id = ui.make_persistent_id(node.id);
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| if ui.button(node.name.clone()).clicked() {})
        .body(|ui| {
            for child_node in &node.childs {
                level_node(ui, child_node, event);
            }
        });
}

pub fn draw(
    context: &Context,
    is_open: &mut bool,
    level: &crate::data_source::Level,
) -> Option<EClickEventType> {
    let mut event: Option<EClickEventType> = None;
    Window::new(format!("Level({})", level.name))
        .open(is_open)
        .show(context, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for node in &level.nodes {
                    level_node(ui, node, &mut event);
                }
            });
        });
    event
}
