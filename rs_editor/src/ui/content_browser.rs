use crate::content_folder::{ContentFolder, EContentFileType};
use egui::{Color32, Context, RichText, Ui};
use std::{cell::RefCell, path::Path, rc::Rc};

pub struct DataSource {
    pub is_open: bool,
    pub current_folder: Option<Rc<RefCell<ContentFolder>>>,
    pub highlight_file: Option<EContentFileType>,
    pub new_folder_name: String,
}

impl DataSource {
    pub fn new() -> Self {
        Self {
            is_open: true,
            current_folder: None,
            new_folder_name: "Untitled".to_string(),
            highlight_file: None,
        }
    }
}

#[derive(Debug)]
pub enum EClickEventType {
    CreateFolder,
    OpenFolder(Rc<RefCell<ContentFolder>>),
    OpenFile(EContentFileType),
    SingleClickFile(EContentFileType),
    Back,
}

enum EItemType {
    Folder(Rc<RefCell<ContentFolder>>),
    File(EContentFileType),
}

pub fn draw(
    window: egui::Window,
    context: &Context,
    asset_folder_path: &Path,
    data_source: &mut DataSource,
) -> Option<EClickEventType> {
    let mut click: Option<EClickEventType> = None;
    let mut click_back: Option<EClickEventType> = None;
    let mut click_item: Option<EClickEventType> = None;
    let open = &mut data_source.is_open;
    let current_folder = data_source.current_folder.clone();
    window
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([500.0, 250.0])
        .show(context, |ui| {
            ui.vertical(|ui| {
                ui.set_max_height(250.0);
                ui.set_max_width(500.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new("Back").color(Color32::WHITE))
                        .clicked()
                    {
                        click_back = Some(EClickEventType::Back);
                    }
                    let response = ui.allocate_response(ui.available_size(), egui::Sense::click());
                    response.context_menu(|ui| {
                        ui.menu_button("Create Folder", |ui| {
                            ui.text_edit_singleline(&mut data_source.new_folder_name);
                            if ui.button("Ok").clicked() {
                                click = Some(EClickEventType::CreateFolder);
                                ui.close_menu();
                            }
                        });
                    });
                });
                if let Some(current_folder) = current_folder {
                    click_item = draw_content(ui, asset_folder_path, current_folder);
                }
                ui.allocate_space(ui.available_size());
            });
        });

    click.or(click_back).or(click_item)
}

fn draw_content(
    ui: &mut Ui,
    asset_folder_path: &Path,
    current_folder: Rc<RefCell<ContentFolder>>,
) -> Option<EClickEventType> {
    let folders = current_folder.borrow().folders.clone();
    let files = current_folder.borrow().files.clone();
    let mut total_items: Vec<EItemType> = vec![];
    for folder in folders {
        total_items.push(EItemType::Folder(folder));
    }
    for file in files {
        total_items.push(EItemType::File(file));
    }
    let mut click: Option<EClickEventType> = None;

    let mut iter = total_items.chunks(10);
    while let Some(row) = iter.next() {
        ui.horizontal_wrapped(|ui| {
            for item in row {
                match item {
                    EItemType::Folder(folder) => {
                        ui.push_id(folder.borrow().name.clone(), |ui| {
                            let response = ui
                                .vertical(|ui| {
                                    ui.set_max_height(50.0);
                                    ui.set_max_width(50.0);
                                    ui.image(egui::include_image!(
                                        "../../../Resource/Editor/folder.svg"
                                    ));
                                    ui.label(folder.borrow().name.clone());
                                })
                                .response;
                            let response = response.interact(egui::Sense::click());
                            if response.double_clicked() {
                                click = Some(EClickEventType::OpenFolder(folder.clone()));
                            }
                        });
                    }
                    EItemType::File(file) => match file {
                        EContentFileType::StaticMesh(static_mesh) => {
                            ui.vertical(|ui| {
                                ui.set_max_height(50.0);
                                ui.set_max_width(50.0);
                                ui.image(egui::include_image!(
                                    "../../../Resource/Editor/static_mesh.svg"
                                ));
                                ui.label(static_mesh.borrow().asset_reference_name.clone());
                            });
                        }
                        EContentFileType::SkeletonMesh(skeleton_mesh) => {
                            ui.vertical(|ui| {
                                ui.set_max_height(50.0);
                                ui.set_max_width(50.0);
                                ui.image(egui::include_image!(
                                    "../../../Resource/Editor/skeleton_mesh.svg"
                                ));
                                ui.label(skeleton_mesh.borrow().get_name().clone());
                            });
                        }
                        EContentFileType::SkeletonAnimation(node_animation) => {
                            ui.vertical(|ui| {
                                ui.set_max_height(50.0);
                                ui.set_max_width(50.0);
                                ui.image(egui::include_image!(
                                    "../../../Resource/Editor/animation.svg"
                                ));
                                ui.label(node_animation.borrow().name.clone());
                            });
                        }
                        EContentFileType::Skeleton(skeleton) => {
                            ui.vertical(|ui| {
                                ui.set_max_height(50.0);
                                ui.set_max_width(50.0);
                                ui.image(egui::include_image!(
                                    "../../../Resource/Editor/skeleton.svg"
                                ));
                                ui.label(skeleton.borrow().get_name().clone());
                            });
                        }
                        EContentFileType::Texture(texture_file) => {
                            let id = texture_file.borrow().name.clone();
                            ui.push_id(id, |ui| {
                                ui.vertical(|ui| {
                                    ui.set_max_height(50.0);
                                    ui.set_max_width(50.0);
                                    if let Some(image_reference) =
                                        texture_file.borrow().image_reference.as_ref()
                                    {
                                        let url = format!(
                                            "file://{}",
                                            asset_folder_path
                                                .join(image_reference)
                                                .to_str()
                                                .unwrap()
                                        );
                                        ui.image(url);
                                    }
                                    let response = ui.button(texture_file.borrow().name.clone());
                                    if response.clicked() {
                                        click =
                                            Some(EClickEventType::SingleClickFile(file.clone()));
                                    }
                                    if response.double_clicked() {
                                        click = Some(EClickEventType::OpenFile(file.clone()));
                                    }
                                });
                            });
                        }
                    },
                }
            }
        });
    }
    click
}
