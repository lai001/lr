use crate::content_edit::{ContentEditable, UIContentPropertyEvent, UIEvent};
use crate::load_content::types::{PreLoadingContext, SceneWrapper};
use crate::ui::content_item_property_view::ContentItemPropertyView;
use rs_artifact::material_paramenters::{BaseDataValueType, StructField};
use rs_content::TypedContent;
use rs_core_minimal::name_generator::NameGenerator;
use rs_engine::build_content_file_url;
use rs_engine::content::material_paramenters_collection::MaterialParamentersCollection;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_localization::t;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

enum EEventType {
    Update(MaterialParamentersCollectionUpdated),
}

struct MaterialParamentersCollectionUpdated {
    fields: Vec<StructField>,
}

impl UIEvent for EEventType {}
impl UIContentPropertyEvent for EEventType {}

pub(super) struct MaterialParamentersCollectionContentEditable {}

fn get_base_data_type_text(base_data_type: &BaseDataValueType) -> String {
    match base_data_type {
        BaseDataValueType::F32(_) => "float32".to_string(),
        BaseDataValueType::Vec2(_) => format!("Vec{}", 2),
        BaseDataValueType::Vec3(_) => format!("Vec{}", 3),
        BaseDataValueType::Vec4(_) => format!("Vec{}", 4),
    }
}

impl ContentEditable for MaterialParamentersCollectionContentEditable {
    fn render_thumbnail(
        &self,
        content: rs_foundation::new::SingleThreadMutType<Box<dyn rs_content::Content>>,
        project_folder_path: &std::path::Path,
        thumbnail_cache: &mut crate::thumbnail_cache::ThumbnailCache,
        expected_thumbnail_render_szie: egui::Vec2,
        ui: &mut egui::Ui,
    ) {
        let _ = content;
        let _ = project_folder_path;
        let _ = thumbnail_cache;
        let _ = expected_thumbnail_render_szie;
        ui.image(egui::include_image!("../../../Resource/Editor/file.svg"));
    }

    fn render_detail(
        &self,
        content: rs_foundation::new::SingleThreadMutType<Box<dyn rs_content::Content>>,
        content_item_property_view: &mut ContentItemPropertyView,
        ui: &mut egui::Ui,
    ) -> Option<Box<dyn UIContentPropertyEvent>> {
        let _ = content_item_property_view;
        let collection_content =
            TypedContent::<MaterialParamentersCollection>::new(content).ok()?;
        let mut collection = collection_content.borrow_mut();

        let mut is_need_update = false;
        let mut delete_field_index: Option<usize> = None;

        let is_add = ui
            .button(egui::WidgetText::RichText(Arc::new(
                egui::RichText::new(t!("Add Field")).strong(),
            )))
            .clicked();
        if is_add {
            let names = collection.fields.iter().map(|x| x.name.clone()).collect();
            let mut generator = NameGenerator::new(names);
            let new_name = generator.next("field");
            collection.fields.push(StructField {
                name: new_name,
                data_type: BaseDataValueType::F32(0.0),
            });
            is_need_update = true;
        }

        for (index, field) in collection.fields.iter_mut().enumerate() {
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
                                    .selectable_value(&mut field.data_type, selected_value, text)
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
                        egui::RichText::new(t!("Remove Field")).strong(),
                    )))
                    .clicked();
                if is_delete {
                    delete_field_index = Some(index);
                }
            });
        }

        if let Some(delete_field_index) = delete_field_index {
            collection.fields.remove(delete_field_index);
            is_need_update = true;
        }

        if is_need_update {
            return Some(Box::new(EEventType::Update(
                MaterialParamentersCollectionUpdated {
                    fields: collection.fields.clone(),
                },
            )));
        }

        None
    }

    fn on_process_event(
        &self,
        editor_context: &mut crate::editor_context::EditorContext,
        content: rs_foundation::new::SingleThreadMutType<Box<dyn rs_content::Content>>,
        event: Box<dyn UIContentPropertyEvent>,
    ) {
        let Some(event) = event.downcast_ref::<EEventType>() else {
            return;
        };
        match event {
            EEventType::Update(update_info) => {
                let mut content_guard = content.borrow_mut();
                if let Some(collection) =
                    content_guard.downcast_mut::<MaterialParamentersCollection>()
                {
                    collection.fields = update_info.fields.clone();
                    collection.initialize(editor_context.engine_mut());
                }
            }
        }
    }

    fn load_sync<'a>(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        loading_context: PreLoadingContext<'a>,
        scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>>,
        engine: &mut rs_engine::engine::Engine,
    ) {
        let _ = scenes;
        let _ = loading_context;
        let collection =
            TypedContent::<MaterialParamentersCollection>::new(content).expect("Matched type");
        let mut collection = collection.borrow_mut();
        collection.initialize(engine);
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn rs_artifact_types::asset::Asset>>,
        model_loader: &mut rs_model_loader::model_loader::ModelLoader,
        project_context: &crate::project_context::ProjectContext,
    ) -> anyhow::Result<()> {
        let _ = associated_assets;
        let _ = model_loader;
        let _ = project_context;
        let collection =
            TypedContent::<MaterialParamentersCollection>::new(content).expect("Matched type");
        let material_paramenters_collection = collection.borrow();
        artifact_asset_encoder.encode_content(&*material_paramenters_collection);
        Ok(())
    }

    fn create_default(
        &self,
        name: String,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let _ = editor_context;
        let content_url = build_content_file_url(&name).ok()?;
        let material_paramenters_collection = MaterialParamentersCollection::new(content_url);
        Some(Box::new(material_paramenters_collection))
    }

    fn display_name_for_creation(&self) -> Option<std::borrow::Cow<'static, str>> {
        Some(t!("Material Parameters Collection"))
    }
}
