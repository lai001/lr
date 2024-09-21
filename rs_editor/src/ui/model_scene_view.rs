use egui::Ui;
use rs_assimp::scene::Scene;
use rs_foundation::new::SingleThreadMutType;

use super::object_property_view::ObjectPropertyView;

enum EEventType {
    ClickNode(String),
}

#[derive(Default)]
pub struct DataSource {
    pub model_scene: Option<std::path::PathBuf>,
    pub selected_node_path: Option<String>,
}

fn draw_node(
    ui: &mut Ui,
    node: SingleThreadMutType<rs_assimp::node::Node<'_>>,
    event: &mut Option<EEventType>,
) {
    let name = { node.borrow().name.clone() };
    let id = ui.make_persistent_id(name.clone());
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            let response = ui.button(name);
            if response.clicked() {
                *event = Some(EEventType::ClickNode(node.borrow().path.clone()));
            }
        })
        .body(|ui| {
            for child in &node.borrow().children {
                draw_node(ui, child.clone(), event);
            }
        });
}

pub fn render(ui: &mut Ui, scene: &Scene, data_source: &mut DataSource) {
    ui.label(format!("Name: {}", scene.name.clone()));
    let mut event: Option<EEventType> = None;
    let Some(root_node) = scene.root_node.clone() else {
        return;
    };
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            draw_node(ui, root_node, &mut event);
        });
        ui.vertical(|ui| {
            if let Some(node_path) = &data_source.selected_node_path {
                let node = scene.all_nodes[node_path].clone();
                render_node(ui, node);
            }

            let Some(event) = event else {
                return;
            };
            match event {
                EEventType::ClickNode(node_path) => {
                    data_source.selected_node_path = Some(node_path.clone());
                }
            }
        });
    });

    for animation in &scene.animations {
        ui.label(format!(
            "Animation: {}, {}",
            animation.name, animation.duration
        ));
    }

    for (name, _) in &scene.armatures {
        ui.label(format!("Armature: {}", name));
    }

    for skeleton in &scene.skeletons {
        ui.label(format!("Skeleton: {}", skeleton.name));
    }
}

fn render_node(ui: &mut Ui, node: SingleThreadMutType<rs_assimp::node::Node<'_>>) {
    let node = node.borrow();
    ObjectPropertyView::transformation_detail(&node.transformation, ui);
    for mesh in node.meshes.clone() {
        let mesh = mesh.borrow();
        ui.label(format!("Mesh: {}", mesh.name));
    }
}
