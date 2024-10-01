use egui::Ui;
use rs_assimp::scene::Scene;
use rs_foundation::new::SingleThreadMutType;

use super::object_property_view::ObjectPropertyView;

enum EEventType {
    ClickNode(String),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum EPageType {
    Scene,
    Mesh,
}

impl Default for EPageType {
    fn default() -> Self {
        EPageType::Scene
    }
}

enum EColumnType {
    Index,
    Vertex,
    TextureCoord(usize),
}

impl ToString for EColumnType {
    fn to_string(&self) -> String {
        match self {
            EColumnType::Index => "Index".to_string(),
            EColumnType::Vertex => "Vertex".to_string(),
            EColumnType::TextureCoord(index) => {
                format!("TextureCoord{}", index)
            }
        }
    }
}

#[derive(Default)]
pub struct DataSource {
    pub model_scene: Option<std::path::PathBuf>,
    pub selected_node_path: Option<String>,
    pub selected_mesh_name: Option<String>,
    pub selected_bond_path: Option<String>,
    pub page_type: EPageType,
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

    ui.horizontal(|ui| {
        ui.radio_value(&mut data_source.page_type, EPageType::Scene, "Scene");
        ui.radio_value(&mut data_source.page_type, EPageType::Mesh, "Mesh");
    });

    match data_source.page_type {
        EPageType::Scene => {
            render_scene_page(ui, scene, data_source, &mut event);
        }
        EPageType::Mesh => {
            render_mesh_page(ui, scene, data_source, &mut event);
        }
    }
}

fn render_mesh_page(
    ui: &mut Ui,
    scene: &Scene,
    data_source: &mut DataSource,
    event: &mut Option<EEventType>,
) {
    let _ = event;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            for mesh in &scene.meshes {
                let mesh = mesh.borrow();
                let name = mesh.name.clone();
                let response = ui.button(&name);
                if response.clicked() {
                    data_source.selected_mesh_name = Some(name);
                }
            }
        });
        ui.set_min_height(500.0);

        ui.vertical(|ui| {
            let selected_mesh = scene.meshes.iter().find(|x| {
                let mesh = x.borrow();
                Some(&mesh.name) == data_source.selected_mesh_name.as_ref()
            });
            let Some(selected_mesh) = selected_mesh else {
                return;
            };
            let selected_mesh = selected_mesh.borrow();

            let total_rows = selected_mesh.vertices.len();
            let text_height = egui::TextStyle::Body
                .resolve(ui.style())
                .size
                .max(ui.spacing().interact_size.y);
            let available_height = ui.available_height();

            let mut column_types: Vec<EColumnType> = vec![];
            column_types.push(EColumnType::Index);
            column_types.push(EColumnType::Vertex);

            for map_index in 0..selected_mesh.texture_coords.len() {
                column_types.push(EColumnType::TextureCoord(map_index));
            }

            let mut table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height);

            for _ in &column_types {
                table = table.column(egui_extras::Column::auto());
            }

            table
                .header(20.0, |mut header| {
                    for column_type in &column_types {
                        header.col(|ui| {
                            ui.strong(column_type.to_string());
                        });
                    }
                })
                .body(|body| {
                    body.rows(text_height, total_rows, |mut row| {
                        let row_index = row.index();

                        for column_type in &column_types {
                            match column_type {
                                EColumnType::Index => {
                                    row.col(|ui| {
                                        ui.label(row_index.to_string());
                                    });
                                }
                                EColumnType::Vertex => {
                                    let vertex = selected_mesh.vertices[row_index];
                                    row.col(|ui| {
                                        ui.label(vertex.to_string());
                                    });
                                }
                                EColumnType::TextureCoord(map_index) => {
                                    let texture_coord_map =
                                        &selected_mesh.texture_coords[*map_index];
                                    row.col(|ui| {
                                        ui.label(texture_coord_map[row_index].to_string());
                                    });
                                }
                            }
                        }
                    });
                });
        });
    });
}

fn render_scene_page(
    ui: &mut Ui,
    scene: &Scene,
    data_source: &mut DataSource,
    event: &mut Option<EEventType>,
) {
    let Some(root_node) = scene.root_node.clone() else {
        return;
    };
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            draw_node(ui, root_node, event);
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
    ui.label("Transformation");
    ObjectPropertyView::transformation_detail(&node.transformation, ui);
    for mesh in node.meshes.clone() {
        let mesh = mesh.borrow();
        ui.label(format!("Mesh: {}", mesh.name));
        for bone in &mesh.bones {
            ui.label(format!("Bone: {}", bone.borrow().name));
        }
    }
}
