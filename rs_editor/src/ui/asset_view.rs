use crate::{
    content_edit::ContentEdit,
    data_source::{AssetFile, AssetFolder},
    thumbnail_cache::ThumbnailCache,
};
use egui::{Color32, Context, RichText, Ui};
use rs_engine::file_type::EFileType;
use rs_localization::t;
use std::any::TypeId;

#[derive(Debug)]
pub enum EClickItemType {
    Folder(AssetFolder),
    File(AssetFile),
    SingleClickFile(AssetFile),
    PlaySound(AssetFile),
    ImportAsActor(AssetFile),
    Back,
    CreateContent(TypeId, AssetFile),
}

pub fn draw(
    window: egui::Window,
    context: &Context,
    open: &mut bool,
    asset_folder: Option<&AssetFolder>,
    highlight_file: Option<&AssetFile>,
    thumbnail_cache: &mut ThumbnailCache,
    content_edit: &mut ContentEdit,
) -> Option<EClickItemType> {
    let mut click_back: Option<EClickItemType> = None;
    let mut click_asset: Option<EClickItemType> = None;
    window
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([350.0, 150.0])
        .show(context, |ui| {
            // ui.set_max_height(250.0);
            // ui.set_max_width(500.0);
            if let Some(asset_folder) = &asset_folder {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new(t!("Back")).color(Color32::WHITE))
                            .clicked()
                        {
                            click_back = Some(EClickItemType::Back);
                        }
                        ui.label(asset_folder.path.to_str().unwrap());
                    });
                    ui.separator();
                    let rect = ui.available_rect_before_wrap();
                    egui::ScrollArea::both().show_viewport(ui, |ui, _| {
                        let mut menu_child_ui = ui.new_child(egui::UiBuilder::new());
                        let response = menu_child_ui.allocate_rect(rect, egui::Sense::click());
                        response.context_menu(|ui| {
                            ui.menu_button(
                                t!("Create"),
                                |ui| {
                                    if ui.button(t!("Material")).clicked() {}
                                },
                            );
                        });
                        click_asset = draw_content(
                            ui,
                            asset_folder,
                            highlight_file,
                            thumbnail_cache,
                            content_edit,
                        );
                    });
                });
            }
            ui.allocate_space(ui.available_size());
        });
    click_asset.or(click_back)
}

enum EAssetItem<'a> {
    AssetFolder(&'a AssetFolder),
    AssetFile(&'a AssetFile),
}

fn draw_content(
    ui: &mut Ui,
    asset_folder: &AssetFolder,
    highlight_file: Option<&AssetFile>,
    thumbnail_cache: &mut ThumbnailCache,
    content_edit: &mut ContentEdit,
) -> Option<EClickItemType> {
    let mut total_items: Vec<EAssetItem> = vec![];
    for folder in &asset_folder.folders {
        total_items.push(EAssetItem::AssetFolder(&folder));
    }
    for file in &asset_folder.files {
        total_items.push(EAssetItem::AssetFile(&file));
    }
    let chunk_size = ((ui.available_width() / 50.0).floor() as usize - 1).max(1);
    let mut iter = total_items.chunks(chunk_size);
    let mut click_item: Option<EClickItemType> = None;
    let mut highlight_item: Option<EClickItemType> = None;

    while let Some(row) = iter.next() {
        ui.horizontal_wrapped(|ui| {
            for item in row {
                match item {
                    EAssetItem::AssetFolder(folder) => {
                        let folder = *folder;
                        ui.push_id(folder.name.clone(), |ui| {
                            let response = ui
                                .vertical(|ui| {
                                    ui.set_max_height(50.0);
                                    ui.set_max_width(50.0);
                                    ui.image(egui::include_image!(
                                        "../../../Resource/Editor/folder.svg"
                                    ));
                                    ui.label(folder.name.clone());
                                })
                                .response;
                            let response = response.interact(egui::Sense::click());
                            if response.double_clicked() {
                                click_item = Some(EClickItemType::Folder(folder.clone()));
                            }
                        });
                    }
                    EAssetItem::AssetFile(file) => {
                        let mut supported: Vec<(std::any::TypeId, String)> = vec![];
                        for (type_id, editable) in content_edit.editables() {
                            if editable.is_support_asset_file(*file)
                                && let Some(display_name) = editable.display_name_for_creation()
                            {
                                supported.push((type_id.clone(), display_name.to_string()));
                                break;
                            }
                        }

                        let file = *file;
                        ui.push_id(file.name.clone(), |ui| {
                            ui.vertical(|ui| {
                                ui.set_max_height(50.0);
                                ui.set_max_width(50.0);
                                if let Some(highlight_file) = highlight_file {
                                    if highlight_file.path == file.path {
                                        ui.painter().rect_filled(
                                            ui.max_rect(),
                                            0.0,
                                            Color32::LIGHT_BLUE,
                                        );
                                    }
                                }
                                render_thumbnail(file, thumbnail_cache, ui);
                                let response = ui.button(file.name.clone());
                                if response.clicked() {
                                    highlight_item =
                                        Some(EClickItemType::SingleClickFile(file.clone()));
                                }
                                if response.double_clicked() {
                                    click_item = Some(EClickItemType::File(file.clone()));
                                }
                                if supported.is_empty() {
                                    match file.get_file_type() {
                                        EFileType::Fbx
                                        | EFileType::Glb
                                        | EFileType::Blend
                                        | EFileType::Dae => {
                                            response.context_menu(|ui| {
                                                highlight_item = Some(
                                                    EClickItemType::SingleClickFile(file.clone()),
                                                );
                                                if ui.button(t!("Import as actor")).clicked() {
                                                    click_item = Some(
                                                        EClickItemType::ImportAsActor(file.clone()),
                                                    );
                                                    ui.close_kind(egui::UiKind::Menu);
                                                }
                                            });
                                        }
                                        EFileType::WAV
                                        | EFileType::MP3
                                        | EFileType::Jpeg
                                        | EFileType::Jpg
                                        | EFileType::Png
                                        | EFileType::Exr
                                        | EFileType::Hdr
                                        | EFileType::Mp4
                                        | EFileType::Texture(_) => {}
                                    }
                                } else {
                                    response.context_menu(|ui| {
                                        ui.menu_button(t!("Create"), |ui| {
                                            for (id, display_name) in supported {
                                                highlight_item = Some(
                                                    EClickItemType::SingleClickFile(file.clone()),
                                                );
                                                if ui.button(display_name).clicked() {
                                                    click_item =
                                                        Some(EClickItemType::CreateContent(
                                                            id,
                                                            file.clone(),
                                                        ));
                                                    ui.close_kind(egui::UiKind::Menu);
                                                }
                                                break;
                                            }
                                        });
                                    });
                                }
                            });
                        });
                    }
                }
            }
        });
    }

    let item = click_item.or(highlight_item);
    item
}

fn render_thumbnail(file: &AssetFile, thumbnail_cache: &mut ThumbnailCache, ui: &mut Ui) {
    let thumbnail_render_szie = egui::vec2(50.0, 50.0);
    match file.get_file_type() {
        EFileType::Fbx | EFileType::Glb | EFileType::Blend | EFileType::Dae => {
            ui.image(egui::include_image!("../../../Resource/Editor/model.svg"));
        }
        EFileType::Jpeg | EFileType::Jpg | EFileType::Png | EFileType::Exr | EFileType::Hdr => {
            match thumbnail_cache.get_image_file_uri(&file.path) {
                Some(uri) => {
                    // ui.image(uri);
                    ui.add_sized(thumbnail_render_szie, egui::Image::new(uri));
                }
                None => {
                    thumbnail_cache.load_image(&file.path);
                    ui.add_sized(
                        thumbnail_render_szie,
                        egui::Spinner::new().size(thumbnail_render_szie.x),
                    );
                }
            }
        }
        EFileType::Mp4 => {
            ui.painter_at(ui.available_rect_before_wrap()).rect_filled(
                ui.available_rect_before_wrap(),
                0.0,
                Color32::WHITE,
            );
            ui.allocate_space(ui.available_rect_before_wrap().size());
        }
        EFileType::WAV | EFileType::MP3 => {
            ui.image(egui::include_image!("../../../Resource/Editor/sound.svg"));
        }
        EFileType::Texture(_) => unimplemented!(),
    }
}
