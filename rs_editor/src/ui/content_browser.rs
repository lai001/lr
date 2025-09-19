use crate::{content_folder::ContentFolder, thumbnail_cache::ThumbnailCache};
use egui::{Color32, Context, RichText, Sense, Ui};
use rs_engine::content::content_file_type::EContentFileType;
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use std::{cell::RefCell, path::Path, rc::Rc};

pub struct DataSource {
    pub is_open: bool,
    pub contents: SingleThreadMutType<Vec<EContentFileType>>,
    pub current_folder: Option<Rc<RefCell<ContentFolder>>>,
    pub highlight_file: Option<EContentFileType>,
    pub new_folder_name: String,
    pub new_material_name: String,
    pub new_ibl_name: String,
    pub new_content_name: String,
    pub new_level_name: String,
}

impl DataSource {
    pub fn new() -> Self {
        Self {
            is_open: true,
            current_folder: None,
            new_folder_name: "Untitled".to_string(),
            highlight_file: None,
            new_material_name: "Untitled".to_string(),
            new_ibl_name: "Untitled".to_string(),
            new_content_name: "Untitled".to_string(),
            new_level_name: "Untitled".to_string(),
            contents: SingleThreadMut::new(vec![]),
        }
    }
}

pub enum EClickEventType {
    CreateFolder,
    CreateMaterial,
    CreateIBL,
    CreateParticleSystem,
    CreateCurve,
    CreateBlendAnimations,
    CreateLevel,
    CreateMaterialParametersCollection,
    OpenFolder(Rc<RefCell<ContentFolder>>),
    OpenFile(EContentFileType),
    DeleteFile(EContentFileType),
    SingleClickFile(EContentFileType),
    Back,
    Rename(EContentFileType, String),
    Detail(EContentFileType),
}

enum EItemType {
    Folder(Rc<RefCell<ContentFolder>>),
    File(EContentFileType),
}

pub fn draw(
    window: egui::Window,
    context: &Context,
    project_folder_path: &Path,
    data_source: &mut DataSource,
    thumbnail_cache: &mut ThumbnailCache,
) -> Option<EClickEventType> {
    let mut click: Option<EClickEventType> = None;
    let mut click_back: Option<EClickEventType> = None;
    let mut click_item: Option<EClickEventType> = None;
    let open = &mut data_source.is_open;
    let current_folder = data_source.current_folder.clone();
    let highlight = data_source.highlight_file.clone();
    window
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([500.0, 250.0])
        .show(context, |ui| {
            ui.vertical(|ui| {
                // ui.set_max_height(250.0);
                // ui.set_max_width(500.0);
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
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        });
                        ui.menu_button("Material", |ui| {
                            ui.text_edit_singleline(&mut data_source.new_material_name);
                            if ui.button("Ok").clicked() {
                                click = Some(EClickEventType::CreateMaterial);
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        });
                        ui.menu_button("IBL", |ui| {
                            ui.text_edit_singleline(&mut data_source.new_ibl_name);
                            if ui.button("Ok").clicked() {
                                click = Some(EClickEventType::CreateIBL);
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        });
                        ui.menu_button("Particle System", |ui| {
                            ui.text_edit_singleline(&mut data_source.new_content_name);
                            if ui.button("Ok").clicked() {
                                click = Some(EClickEventType::CreateParticleSystem);
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        });
                        ui.menu_button("Curve", |ui| {
                            ui.text_edit_singleline(&mut data_source.new_content_name);
                            if ui.button("Ok").clicked() {
                                click = Some(EClickEventType::CreateCurve);
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        });
                        ui.menu_button("Blend Animation", |ui| {
                            ui.text_edit_singleline(&mut data_source.new_content_name);
                            if ui.button("Ok").clicked() {
                                click = Some(EClickEventType::CreateBlendAnimations);
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        });
                        ui.menu_button("Material Parameters Collection", |ui| {
                            ui.text_edit_singleline(&mut data_source.new_content_name);
                            if ui.button("Ok").clicked() {
                                click = Some(EClickEventType::CreateMaterialParametersCollection);
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        });
                        ui.menu_button("Level", |ui| {
                            ui.text_edit_singleline(&mut data_source.new_level_name);
                            if ui.button("Ok").clicked() {
                                click = Some(EClickEventType::CreateLevel);
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        });
                    });
                });
                if let Some(current_folder) = current_folder {
                    click_item = draw_content(
                        ui,
                        project_folder_path,
                        current_folder,
                        highlight,
                        thumbnail_cache,
                    );
                }
                ui.allocate_space(ui.available_size());
            });
        });

    click.or(click_back).or(click_item)
}

fn draw_content(
    ui: &mut Ui,
    project_folder_path: &Path,
    current_folder: Rc<RefCell<ContentFolder>>,
    highlight_file: Option<EContentFileType>,
    thumbnail_cache: &mut ThumbnailCache,
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

    let chunk_size = ((ui.available_width() / 50.0).floor() as usize - 1).max(1);
    let mut iter = total_items.chunks(chunk_size);
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
                    EItemType::File(file) => {
                        let name = file.get_name().clone();
                        let url = file.get_url();
                        ui.vertical(|ui| {
                            ui.set_max_height(50.0);
                            ui.set_max_width(50.0);
                            let mut response = ui
                                .push_id(name.clone(), |ui| {
                                    if let Some(highlight_file) = highlight_file.as_ref() {
                                        if highlight_file.get_url() == file.get_url() {
                                            ui.painter().rect_filled(
                                                ui.available_rect_before_wrap(),
                                                0.0,
                                                Color32::LIGHT_BLUE,
                                            );
                                        }
                                    }
                                    render_thumbnail(
                                        file,
                                        project_folder_path,
                                        thumbnail_cache,
                                        ui,
                                    );
                                })
                                .response;
                            response = response.interact(Sense::click_and_drag());
                            if response.clicked() {
                                click = Some(EClickEventType::SingleClickFile(file.clone()));
                            }
                            if response.double_clicked() {
                                click = Some(EClickEventType::OpenFile(file.clone()));
                            }
                            response.context_menu(|ui| {
                                if ui.button("Detail").clicked() {
                                    click = Some(EClickEventType::Detail(file.clone()));
                                    ui.close_kind(egui::UiKind::Menu);
                                }
                                if ui.button("Copy Reference").clicked() {
                                    ui.ctx().copy_text(url.to_string());
                                    // ui.output_mut(|p| p.copied_text = url.to_string());
                                    ui.close_kind(egui::UiKind::Menu);
                                }
                                if ui.button("Delete").clicked() {
                                    click = Some(EClickEventType::DeleteFile(file.clone()));
                                    ui.close_kind(egui::UiKind::Menu);
                                }
                            });
                            let mut edit_name = name.clone();
                            if ui.text_edit_multiline(&mut edit_name).changed() {
                                click = Some(EClickEventType::Rename(file.clone(), edit_name));
                            }
                        });
                    }
                }
            }
        });
    }
    click
}

fn render_thumbnail(
    file: &EContentFileType,
    project_folder_path: &Path,
    thumbnail_cache: &mut ThumbnailCache,
    ui: &mut Ui,
) {
    let thumbnail_render_szie = egui::vec2(50.0, 50.0);
    match file {
        EContentFileType::StaticMesh(_) => {
            ui.image(egui::include_image!(
                "../../../Resource/Editor/static_mesh.svg"
            ));
        }
        EContentFileType::SkeletonMesh(_) => {
            ui.image(egui::include_image!(
                "../../../Resource/Editor/skeleton_mesh.svg"
            ));
        }
        EContentFileType::SkeletonAnimation(_) => {
            ui.image(egui::include_image!(
                "../../../Resource/Editor/animation.svg"
            ));
        }
        EContentFileType::Skeleton(_) => {
            ui.image(egui::include_image!(
                "../../../Resource/Editor/skeleton.svg"
            ));
        }
        EContentFileType::Texture(texture) => {
            if let Some(image_reference) = texture.borrow().get_image_reference_path().as_ref() {
                let path = project_folder_path.join(image_reference);
                match thumbnail_cache.get_image_file_uri(&path) {
                    Some(uri) => {
                        ui.add_sized(thumbnail_render_szie, egui::Image::new(uri));
                    }
                    None => {
                        thumbnail_cache.load_image(&path);
                        ui.add_sized(
                            thumbnail_render_szie,
                            egui::Spinner::new().size(thumbnail_render_szie.x),
                        );
                    }
                }
            }
        }
        EContentFileType::Level(_) => {
            ui.image(egui::include_image!("../../../Resource/Editor/level.svg"));
        }
        EContentFileType::Material(_) => {
            ui.image(egui::include_image!(
                "../../../Resource/Editor/material.svg"
            ));
        }
        EContentFileType::IBL(ibl) => {
            let ibl = ibl.borrow();
            if let Some(image_reference) = &ibl.image_reference {
                let path = project_folder_path.join(image_reference);
                match thumbnail_cache.get_image_file_uri(&path) {
                    Some(uri) => {
                        ui.add_sized(thumbnail_render_szie, egui::Image::new(uri));
                    }
                    None => {
                        thumbnail_cache.load_image(&path);
                        ui.add_sized(
                            thumbnail_render_szie,
                            egui::Spinner::new().size(thumbnail_render_szie.x),
                        );
                    }
                }
            }
        }
        EContentFileType::ParticleSystem(_) => {
            ui.image(egui::include_image!(
                "../../../Resource/Editor/particle.svg"
            ));
        }
        EContentFileType::Sound(_) => {
            ui.image(egui::include_image!("../../../Resource/Editor/sound.svg"));
        }
        EContentFileType::Curve(_) => {
            ui.image(egui::include_image!("../../../Resource/Editor/curve.svg"));
        }
        EContentFileType::BlendAnimations(_) => {
            ui.image(egui::include_image!(
                "../../../Resource/Editor/blend_animations.svg"
            ));
        }
        EContentFileType::MaterialParamentersCollection(_) => {
            ui.image(egui::include_image!("../../../Resource/Editor/file.svg"));
        }
    }
}
