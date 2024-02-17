use crate::texture::{TextureFile, TextureFolder};
use egui::{Color32, Context, RichText, Ui, Window};
use std::{cell::RefCell, path::Path, rc::Rc};

#[derive(Debug)]
pub enum EClickItemType {
    Folder(TextureFolder),
    File(Rc<RefCell<TextureFile>>),
    SingleClickFile(Rc<RefCell<TextureFile>>),
    CreateTexture(TextureFile),
    CreateTextureFolder(TextureFolder),
    Back,
}

pub struct DataSource {
    pub is_textures_view_open: bool,
    pub texture_folder: Option<TextureFolder>,
    pub current_texture_folder: Option<TextureFolder>,
    pub highlight_texture_file: Option<Rc<RefCell<TextureFile>>>,
}

impl DataSource {
    pub fn new() -> Self {
        Self {
            is_textures_view_open: true,
            texture_folder: None,
            current_texture_folder: None,
            highlight_texture_file: None,
        }
    }
}

pub fn draw(
    context: &Context,
    open: &mut bool,
    asset_folder_path: &Path,
    textures_folder: Option<&TextureFolder>,
    highlight_file: Option<Rc<RefCell<TextureFile>>>,
) -> Option<EClickItemType> {
    let mut click_back: Option<EClickItemType> = None;
    let mut click_texture: Option<EClickItemType> = None;
    let mut click_texture_folder: Option<EClickItemType> = None;
    Window::new("Textures")
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([350.0, 150.0])
        .show(context, |ui| {
            if let Some(textures_folder) = textures_folder {
                ui.vertical(|ui| {
                    ui.set_max_height(250.0);
                    ui.set_max_width(500.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new("Back").color(Color32::WHITE))
                            .clicked()
                        {
                            click_back = Some(EClickItemType::Back);
                        }
                        if ui
                            .button(RichText::new("Create Texture Folder").color(Color32::WHITE))
                            .clicked()
                        {
                            click_back =
                                Some(EClickItemType::CreateTextureFolder(textures_folder.clone()));
                        }
                        ui.label(textures_folder.url.to_string());
                    });
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        click_texture =
                            draw_content(ui, asset_folder_path, textures_folder, highlight_file);
                    });
                    ui.allocate_space(ui.available_size());
                });
            }
        });
    click_texture.or(click_back)
}

fn draw_content(
    ui: &mut Ui,
    asset_folder_path: &Path,
    textures_folder: &TextureFolder,
    highlight_file: Option<Rc<RefCell<TextureFile>>>,
) -> Option<EClickItemType> {
    let mut click_item: Option<EClickItemType> = None;
    for folder in &textures_folder.texture_folders {
        ui.push_id(folder.name.clone(), |ui| {
            let response = ui
                .vertical(|ui| {
                    ui.set_max_height(50.0);
                    ui.set_max_width(50.0);
                    ui.image(egui::include_image!("../../../Resource/Editor/folder.svg"));
                    ui.label(folder.name.clone());
                })
                .response;
            let response = response.interact(egui::Sense::click());
            if response.double_clicked() {
                click_item = Some(EClickItemType::Folder(folder.clone()));
            }
        });
    }
    for file in &textures_folder.texture_files {
        let id = file.borrow().name.clone();
        ui.push_id(id, |ui| {
            let highlight_file = highlight_file.clone();
            ui.vertical(|ui| {
                ui.set_max_height(50.0);
                ui.set_max_width(50.0);
                if let Some(highlight_file) = highlight_file {
                    if highlight_file.borrow().url == file.borrow().url {
                        ui.painter()
                            .rect_filled(ui.max_rect(), 0.0, Color32::LIGHT_BLUE);
                    }
                }
                if let Some(image_reference) = file.borrow().image_reference.as_ref() {
                    let url = format!(
                        "file://{}",
                        asset_folder_path.join(image_reference).to_str().unwrap()
                    );
                    ui.image(url);
                }
                let response = ui.button(file.borrow().name.clone());
                if response.clicked() {
                    click_item = Some(EClickItemType::SingleClickFile(file.clone()));
                }
                if response.double_clicked() {
                    click_item = Some(EClickItemType::File(file.clone()));
                }
            });
        });
    }
    click_item
}
