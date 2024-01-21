use crate::data_source::{AssetFolder, DataSource, MeshItem};
use egui::*;
use std::{path::PathBuf, rc::Rc};

#[derive(Debug)]
pub struct ClickMeshItem {
    pub file_path: PathBuf,
    pub item: Rc<MeshItem>,
}

#[derive(Default, Debug)]
pub struct ClickEvent {
    pub is_open_project: bool,
    pub is_new_project: bool,
    pub is_save_project: bool,
    pub is_import_asset: bool,
    pub is_export: bool,
    pub asset_folder: bool,
    pub level_window: bool,
    pub open_visual_studio_code: bool,
    pub open_asset_file_path: Option<PathBuf>,
    pub mesh_item: Option<ClickMeshItem>,
}

pub struct EditorUI {}

impl EditorUI {
    pub fn build(context: &Context, data_source: &mut DataSource) -> ClickEvent {
        let mut click = ClickEvent::default();
        Self::menu(context, data_source, &mut click);

        if data_source.is_asset_folder_open {
            Self::asset_folder(context, data_source, &mut click);
        }
        Self::model_hierarchy_window(context, data_source, &mut click);
        Self::level_window(context, data_source, &mut click);
        let mut is_new_project_window_open = data_source.is_new_project_window_open;
        Window::new("New Project")
            .open(&mut is_new_project_window_open)
            .show(context, |ui| {
                ui.text_edit_singleline(&mut data_source.new_project_name);

                if ui.add(Button::new("OK")).clicked() {
                    click.is_new_project = true;
                    data_source.is_new_project_window_open = false;
                }
            });
        data_source.is_new_project_window_open = is_new_project_window_open;
        click
    }

    fn menu(context: &Context, data_source: &mut DataSource, click: &mut ClickEvent) {
        TopBottomPanel::top("menu_bar").show(context, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.set_min_width(220.0);
                    ui.style_mut().wrap = Some(false);
                    if ui.add(Button::new("New Project")).clicked() {
                        data_source.is_new_project_window_open = true;
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Open Project")).clicked() {
                        click.is_open_project = true;
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Import Asset")).clicked() {
                        click.is_import_asset = true;
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Save Project")).clicked() {
                        click.is_save_project = true;
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Export")).clicked() {
                        click.is_export = true;
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Open Visual Studio Code")).clicked() {
                        click.open_visual_studio_code = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Window", |ui| {
                    if ui.add(Button::new("Asset")).clicked() {
                        click.asset_folder = true;
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Level")).clicked() {
                        click.level_window = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn model_hierarchy_window(
        context: &Context,
        data_source: &mut DataSource,
        click: &mut ClickEvent,
    ) {
        Window::new("Model Hierarchy")
            .open(&mut data_source.is_model_hierarchy_open)
            .show(context, |ui| {
                if let Some(model_view_data) = data_source.model_view_data.as_ref() {
                    Self::render_collapsing_header(
                        ui,
                        &model_view_data.mesh_items,
                        &model_view_data.file_path,
                        click,
                    );
                }
            });
    }

    fn render_collapsing_header(
        ui: &mut Ui,
        mesh_items: &[Rc<MeshItem>],
        file_path: &std::path::Path,
        click: &mut ClickEvent,
    ) {
        for mesh_item in mesh_items {
            CollapsingHeader::new(mesh_item.name.clone()).show(ui, |ui| {
                if ui.button("Add").clicked() {
                    click.mesh_item = Some(ClickMeshItem {
                        item: mesh_item.clone(),
                        file_path: file_path.to_path_buf(),
                    });
                }
                Self::render_collapsing_header(ui, &mesh_item.childs, file_path, click);
            });
        }
    }

    fn asset_folder(context: &Context, data_source: &mut DataSource, click: &mut ClickEvent) {
        Window::new("Asset")
            .open(&mut data_source.is_asset_folder_open)
            .show(context, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    if let Some(asset_folder) = data_source.asset_folder.as_ref() {
                        Self::asset_folder2(ui, asset_folder, click);
                    }
                });
            });
    }

    fn asset_folder2(ui: &mut Ui, asset_folder: &AssetFolder, click: &mut ClickEvent) {
        CollapsingHeader::new(asset_folder.name.clone()).show(ui, |ui| {
            for folder in &asset_folder.folders {
                Self::asset_folder2(ui, folder, click);
            }
            for file in &asset_folder.files {
                if ui.button(file.name.clone()).double_clicked() {
                    click.open_asset_file_path = Some(file.path.clone());
                }
            }
        });
    }

    fn level_node(ui: &mut Ui, node: &crate::data_source::Node, click: &mut ClickEvent) {
        CollapsingHeader::new(node.name.clone())
            .id_source(node.id)
            .show(ui, |ui| {
                for child_node in &node.childs {
                    Self::level_node(ui, child_node, click);
                }
            });
    }

    fn level_window(context: &Context, data_source: &mut DataSource, click: &mut ClickEvent) {
        let Some(level) = data_source.level.as_ref() else {
            return;
        };
        Window::new(format!("Level({})", level.name))
            .open(&mut data_source.is_level_view_open)
            .show(context, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for node in &level.nodes {
                        Self::level_node(ui, node, click);
                    }
                });
            });
    }
}
