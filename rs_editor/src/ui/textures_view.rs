use egui::{Color32, Context, RichText, Ui, Window};
use std::path::Path;

#[derive(Debug)]
pub enum EClickItemType<'a> {
    Folder(&'a crate::texture::TextureFolder),
    File(&'a crate::texture::TextureFile),
    SingleClickFile(&'a crate::texture::TextureFile),
    CreateTexture(&'a crate::texture::TextureFile),
    CreateTextureFolder(&'a crate::texture::TextureFolder),
    Back,
}

pub struct DataSource {
    pub is_textures_view_open: bool,
    pub texture_folder: Option<crate::texture::TextureFolder>,
    pub current_texture_folder: Option<crate::texture::TextureFolder>,
    pub highlight_texture_file: Option<crate::texture::TextureFile>,
}

impl DataSource {
    pub fn new() -> Self {
        Self {
            is_textures_view_open: false,
            texture_folder: None,
            current_texture_folder: None,
            highlight_texture_file: None,
        }
    }
}

pub fn draw<'a>(
    context: &Context,
    open: &mut bool,
    asset_folder_path: &Path,
    textures_folder: Option<&'a crate::texture::TextureFolder>,
    highlight_file: Option<&'a crate::texture::TextureFile>,
) -> Option<EClickItemType<'a>> {
    let mut click_back: Option<EClickItemType<'a>> = None;
    let mut click_texture: Option<EClickItemType<'a>> = None;
    let mut click_texture_folder: Option<EClickItemType<'a>> = None;
    Window::new("Textures")
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([350.0, 150.0])
        .show(context, |ui| {
            if let Some(textures_folder) = &textures_folder {
                let response = ui
                    .vertical(|ui| {
                        ui.set_max_height(250.0);
                        ui.set_max_width(500.0);
                        ui.horizontal(|ui| {
                            if ui
                                .button(RichText::new("Back").color(Color32::WHITE))
                                .clicked()
                            {
                                click_back = Some(EClickItemType::Back);
                            }
                            ui.label(textures_folder.url.to_string());
                        });
                        ui.separator();
                        ui.horizontal_wrapped(|ui| {
                            click_texture = draw_content(
                                ui,
                                asset_folder_path,
                                textures_folder,
                                highlight_file,
                            );
                        });
                        ui.allocate_space(ui.available_size());
                    })
                    .response;
                response.context_menu(|ui| {
                    if ui.button("Create texture folder").clicked() {
                        click_texture_folder =
                            Some(EClickItemType::CreateTextureFolder(textures_folder));
                        ui.close_menu();
                    }
                });
            }
        });
    click_texture.or(click_back)
}

fn draw_content<'a>(
    ui: &mut Ui,
    asset_folder_path: &Path,
    textures_folder: &'a crate::texture::TextureFolder,
    highlight_file: Option<&'a crate::texture::TextureFile>,
) -> Option<EClickItemType<'a>> {
    let mut click_item: Option<EClickItemType<'a>> = None;
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
                click_item = Some(EClickItemType::Folder(folder));
            }
        });
    }
    for file in &textures_folder.texture_files {
        let response = ui
            .push_id(file.name.clone(), |ui| {
                ui.vertical(|ui| {
                    ui.set_max_height(50.0);
                    ui.set_max_width(50.0);
                    if let Some(highlight_file) = highlight_file {
                        if highlight_file.url == file.url {
                            ui.painter()
                                .rect_filled(ui.max_rect(), 0.0, Color32::LIGHT_BLUE);
                        }
                    }
                    if let Some(image_reference) = file.image_reference.as_ref() {
                        let url = format!(
                            "file://{}",
                            asset_folder_path.join(image_reference).to_str().unwrap()
                        );
                        ui.image(url);
                    }
                    ui.label(file.name.clone());
                });
            })
            .response;
        let response = response.interact(egui::Sense::click());
        if response.clicked() {
            click_item = Some(EClickItemType::SingleClickFile(file));
        }
        if response.double_clicked() {
            click_item = Some(EClickItemType::File(file));
        }
    }
    click_item
}
