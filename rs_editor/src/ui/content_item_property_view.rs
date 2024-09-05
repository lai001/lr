use rs_engine::content::{content_file_type::EContentFileType, ibl::IBL, texture::TextureFile};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub enum EClickType {
    IBL(Rc<RefCell<IBL>>, Option<PathBuf>, Option<PathBuf>),
    IsVirtualTexture(Rc<RefCell<TextureFile>>, bool),
    SDF2D(Rc<RefCell<TextureFile>>),
}

pub struct ContentItemPropertyView {
    pub content: Option<EContentFileType>,
    pub image_asset_files: Vec<PathBuf>,
    pub click: Option<EClickType>,
}

impl ContentItemPropertyView {
    pub fn new() -> ContentItemPropertyView {
        ContentItemPropertyView {
            content: None,
            image_asset_files: Vec::new(),
            click: None,
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.click = None;

        let Some(content) = &mut self.content.clone() else {
            return;
        };
        self.render_window_content(content, ui);
    }

    fn render_window_content(&mut self, content: &mut EContentFileType, ui: &mut egui::Ui) {
        ui.label(format!(
            "{} ({})",
            content.get_name(),
            content.get_type_text()
        ));
        ui.label(format!("url: {}", content.get_url().to_string()));

        match content {
            EContentFileType::StaticMesh(_) => {}
            EContentFileType::SkeletonMesh(_) => {}
            EContentFileType::SkeletonAnimation(_) => {}
            EContentFileType::Skeleton(_) => {}
            EContentFileType::Texture(texture_file) => {
                let texture_file_clone = texture_file.clone();
                let mut texture_file = texture_file.borrow_mut();
                if ui
                    .checkbox(&mut texture_file.is_virtual_texture, "Is Virtual Texture")
                    .changed()
                {
                    self.click = Some(EClickType::IsVirtualTexture(
                        texture_file_clone.clone(),
                        texture_file.is_virtual_texture,
                    ));
                }
                if ui.button("SDF 2D").clicked() {
                    self.click = Some(EClickType::SDF2D(texture_file_clone));
                }
            }
            EContentFileType::Level(_) => {}
            EContentFileType::Material(_) => {}
            EContentFileType::IBL(ibl) => {
                let ibl_clone = ibl.clone();
                let mut ibl = ibl.borrow_mut();

                let ibl_bake_info = &mut ibl.bake_info;
                ui.add(
                    egui::DragValue::new(&mut ibl_bake_info.brdf_sample_count)
                        .speed(1)
                        .prefix("BRDF Sample Count: ")
                        .range(1..=8192),
                );
                ui.add(
                    egui::DragValue::new(&mut ibl_bake_info.irradiance_sample_count)
                        .speed(1)
                        .prefix("Irradiance Sample Count: ")
                        .range(1..=8192),
                );
                ui.add(
                    egui::DragValue::new(&mut ibl_bake_info.pre_filter_sample_count)
                        .speed(1)
                        .prefix("Prefilter Sample Count: ")
                        .range(1..=8192),
                );
                ui.add(
                    egui::DragValue::new(&mut ibl_bake_info.brdflutmap_length)
                        .speed(1)
                        .prefix("BRDF Length: ")
                        .range(64..=2048),
                );
                ui.add(
                    egui::DragValue::new(&mut ibl_bake_info.pre_filter_cube_map_max_mipmap_level)
                        .speed(1)
                        .prefix("Prefilter Max Mipmap: ")
                        .range(1..=64),
                );
                ui.add(
                    egui::DragValue::new(&mut ibl_bake_info.irradiance_cube_map_length)
                        .speed(1)
                        .prefix("Irradiance Length: ")
                        .range(4..=8192),
                );
                ui.add(
                    egui::DragValue::new(&mut ibl_bake_info.pre_filter_cube_map_length)
                        .speed(1)
                        .prefix("Prefilter Cube Map Length: ")
                        .range(4..=8192),
                );

                let selected_text = if let Some(image_reference) = &ibl.image_reference {
                    image_reference.to_str().unwrap().to_string()
                } else {
                    "None".to_string()
                };
                egui::ComboBox::from_label("asset")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        let old = ibl.image_reference.clone();
                        if ui
                            .selectable_value(&mut ibl.image_reference, None, "None")
                            .clicked()
                        {
                            self.click =
                                Some(EClickType::IBL(ibl_clone.clone(), old.clone(), None));
                        }
                        for image_asset_file in self.image_asset_files.iter() {
                            if ui
                                .selectable_value(
                                    &mut ibl.image_reference,
                                    Some(image_asset_file.clone()),
                                    image_asset_file.to_str().unwrap(),
                                )
                                .clicked()
                            {
                                self.click = Some(EClickType::IBL(
                                    ibl_clone.clone(),
                                    old.clone(),
                                    Some(image_asset_file.clone()),
                                ));
                            }
                        }
                    });
            }
            EContentFileType::ParticleSystem(_) => {}
            EContentFileType::Sound(_) => {}
        }
    }
}
