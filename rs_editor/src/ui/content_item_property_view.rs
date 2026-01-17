use crate::ui::misc::render_combo_box_not_null;
use rs_core_minimal::name_generator::NameGenerator;
use rs_engine::{
    content::{
        content_file_type::EContentFileType, ibl::IBL,
        material_paramenters_collection::MaterialParamentersCollection,
        render_target_2d::RenderTarget2D, texture::TextureFile,
    },
    uniform_map::{BaseDataValueType, StructField},
};
use rs_foundation::new::SingleThreadMutType;
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

pub enum RenderTarget2DPropertyType {
    Width(u32),
    Height(u32),
    Format(wgpu::TextureFormat),
}

pub enum TextureFilePropertyType {
    IsCompressed(bool),
}

pub enum EEventType {
    IBL(Rc<RefCell<IBL>>, Option<PathBuf>, Option<PathBuf>),
    IsVirtualTexture(Rc<RefCell<TextureFile>>, bool),
    SDF2D(Rc<RefCell<TextureFile>>),
    UpdateMaterialParamentersCollection(
        (
            SingleThreadMutType<MaterialParamentersCollection>,
            MaterialParamentersCollection,
        ),
    ),
    UpdateStaticMeshEnableMultiresolution(
        Rc<RefCell<rs_engine::content::static_mesh::StaticMesh>>,
        bool,
        bool,
    ),
    RenderTarget2D(
        SingleThreadMutType<RenderTarget2D>,
        RenderTarget2DPropertyType,
    ),
    TextureFile(SingleThreadMutType<TextureFile>, TextureFilePropertyType),
}

pub struct ContentItemPropertyView {
    pub content: Option<EContentFileType>,
    pub image_asset_files: Vec<PathBuf>,
    pub click: Option<EEventType>,
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
            EContentFileType::StaticMesh(static_mesh) => {
                let value = static_mesh.clone();
                let static_mesh = static_mesh.borrow();
                let old = static_mesh.is_enable_multiresolution;
                let mut new = static_mesh.is_enable_multiresolution;
                ui.label(format!(
                    "Asset url: {}",
                    static_mesh.asset_info.get_url().to_string()
                ));
                ui.checkbox(&mut new, "Is enable multiresolution");
                if old != new {
                    self.click = Some(EEventType::UpdateStaticMeshEnableMultiresolution(
                        value, old, new,
                    ));
                }
            }
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
                    self.click = Some(EEventType::IsVirtualTexture(
                        texture_file_clone.clone(),
                        texture_file.is_virtual_texture,
                    ));
                }
                if ui.button("SDF 2D").clicked() {
                    self.click = Some(EEventType::SDF2D(texture_file_clone.clone()));
                }

                let mut is_compressed = texture_file.is_compressed;
                if ui.checkbox(&mut is_compressed, "Is compressed").changed() {
                    self.click = Some(EEventType::TextureFile(
                        texture_file_clone,
                        TextureFilePropertyType::IsCompressed(is_compressed),
                    ));
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
                                Some(EEventType::IBL(ibl_clone.clone(), old.clone(), None));
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
                                self.click = Some(EEventType::IBL(
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
            EContentFileType::Curve(_) => {}
            EContentFileType::BlendAnimations(_) => {}
            EContentFileType::MaterialParamentersCollection(material_paramenters_collection) => {
                let material_paramenters_collection_response =
                    material_paramenters_collection.clone();

                let material_paramenters_collection = material_paramenters_collection.borrow();

                let mut material_paramenters_collection = material_paramenters_collection.clone();

                let mut is_need_update = false;
                let mut delete_field_index: Option<usize> = None;

                let is_add = ui
                    .button(egui::WidgetText::RichText(Arc::new(
                        egui::RichText::new("+").strong(),
                    )))
                    .clicked();
                if is_add {
                    let names = material_paramenters_collection
                        .fields
                        .iter()
                        .map(|x| x.name.clone())
                        .collect();
                    let mut generator = NameGenerator::new(names);
                    let new_name = generator.next("field");
                    material_paramenters_collection.fields.push(StructField {
                        name: new_name,
                        data_type: BaseDataValueType::F32(0.0),
                    });
                    is_need_update = true;
                }

                for (index, field) in material_paramenters_collection
                    .fields
                    .iter_mut()
                    .enumerate()
                {
                    ui.horizontal(|ui| {
                        let candidate_items = vec![
                            BaseDataValueType::F32(0.0),
                            BaseDataValueType::Vec2(glam::Vec2::ZERO),
                            BaseDataValueType::Vec3(glam::Vec3::ZERO),
                            BaseDataValueType::Vec4(glam::Vec4::ZERO),
                        ];

                        if ui
                            .add(egui::TextEdit::singleline(&mut field.name))
                            .changed()
                        {
                            is_need_update = true;
                        }

                        {
                            let text = get_base_data_type_text(&field.data_type);

                            egui::ComboBox::from_label(format!("{}", index))
                                .selected_text(text)
                                .show_ui(ui, |ui| {
                                    for selected_value in candidate_items {
                                        let text = get_base_data_type_text(&selected_value);
                                        let is_changed = ui
                                            .selectable_value(
                                                &mut field.data_type,
                                                selected_value,
                                                text,
                                            )
                                            .changed();
                                        if is_changed {
                                            is_need_update = true;
                                        }
                                    }
                                });
                        }

                        match &mut field.data_type {
                            BaseDataValueType::F32(value) => {
                                if ui.add(egui::DragValue::new(value)).changed() {
                                    is_need_update = true;
                                }
                            }
                            BaseDataValueType::Vec2(value) => {
                                if ui.add(egui::DragValue::new(&mut value.x)).changed() {
                                    is_need_update = true;
                                }
                                if ui.add(egui::DragValue::new(&mut value.y)).changed() {
                                    is_need_update = true;
                                }
                            }
                            BaseDataValueType::Vec3(value) => {
                                if ui.add(egui::DragValue::new(&mut value.x)).changed() {
                                    is_need_update = true;
                                }
                                if ui.add(egui::DragValue::new(&mut value.y)).changed() {
                                    is_need_update = true;
                                }
                                if ui.add(egui::DragValue::new(&mut value.z)).changed() {
                                    is_need_update = true;
                                }
                                let mut rgba_unmul = [value.x, value.y, value.z, 1.0];
                                if ui
                                    .color_edit_button_rgba_unmultiplied(&mut rgba_unmul)
                                    .changed()
                                {
                                    value.x = rgba_unmul[0];
                                    value.y = rgba_unmul[1];
                                    value.z = rgba_unmul[2];
                                    is_need_update = true;
                                }
                            }
                            BaseDataValueType::Vec4(value) => {
                                if ui.add(egui::DragValue::new(&mut value.x)).changed() {
                                    is_need_update = true;
                                }
                                if ui.add(egui::DragValue::new(&mut value.y)).changed() {
                                    is_need_update = true;
                                }
                                if ui.add(egui::DragValue::new(&mut value.z)).changed() {
                                    is_need_update = true;
                                }
                                if ui.add(egui::DragValue::new(&mut value.w)).changed() {
                                    is_need_update = true;
                                }
                                let mut rgba_unmul = [value.x, value.y, value.z, value.w];
                                if ui
                                    .color_edit_button_rgba_unmultiplied(&mut rgba_unmul)
                                    .changed()
                                {
                                    value.x = rgba_unmul[0];
                                    value.y = rgba_unmul[1];
                                    value.z = rgba_unmul[2];
                                    value.w = rgba_unmul[3];
                                    is_need_update = true;
                                }
                            }
                        }

                        let is_delete = ui
                            .button(egui::WidgetText::RichText(Arc::new(
                                egui::RichText::new("-").strong(),
                            )))
                            .clicked();
                        if is_delete {
                            delete_field_index = Some(index);
                        }
                    });
                }
                if let Some(delete_field_index) = delete_field_index {
                    material_paramenters_collection
                        .fields
                        .remove(delete_field_index);
                    is_need_update = true;
                }
                if is_need_update {
                    self.click = Some(EEventType::UpdateMaterialParamentersCollection((
                        material_paramenters_collection_response,
                        material_paramenters_collection,
                    )));
                }
            }
            EContentFileType::RenderTarget2D(render_target_2d) => {
                let object = render_target_2d.clone();
                let mut render_target_2d = render_target_2d.borrow_mut();
                let response = ui.add(
                    egui::DragValue::new(&mut render_target_2d.width)
                        .speed(1)
                        .prefix("Width: ")
                        .range(1..=4096 * 4)
                        .update_while_editing(false),
                );
                if response.lost_focus() {
                    self.click = Some(EEventType::RenderTarget2D(
                        object.clone(),
                        RenderTarget2DPropertyType::Width(render_target_2d.width),
                    ))
                }
                let response = ui.add(
                    egui::DragValue::new(&mut render_target_2d.height)
                        .speed(1)
                        .prefix("Height: ")
                        .range(1..=4096 * 4)
                        .update_while_editing(false),
                );
                if response.lost_focus() {
                    self.click = Some(EEventType::RenderTarget2D(
                        object.clone(),
                        RenderTarget2DPropertyType::Height(render_target_2d.height),
                    ))
                }
                let format = &mut render_target_2d.format;
                let candidate_items = vec![
                    wgpu::TextureFormat::Rgba8Unorm,
                    wgpu::TextureFormat::R8Unorm,
                ];
                if render_combo_box_not_null(ui, "Format", format, candidate_items) {
                    self.click = Some(EEventType::RenderTarget2D(
                        object.clone(),
                        RenderTarget2DPropertyType::Format(render_target_2d.format),
                    ))
                }
            }
        }
    }
}

fn get_base_data_type_text(base_data_type: &BaseDataValueType) -> String {
    let text = match base_data_type {
        BaseDataValueType::F32(_) => "float32".to_string(),
        BaseDataValueType::Vec2(_) => format!("Vec{}", 2),
        BaseDataValueType::Vec3(_) => format!("Vec{}", 3),
        BaseDataValueType::Vec4(_) => format!("Vec{}", 4),
    };
    text
}
