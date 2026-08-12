use crate::content_edit::{ContentEditable, UIContentPropertyEvent, UIEvent};
use crate::load_content::types::{PreLoadingContext, SceneWrapper};
use crate::ui::content_item_property_view::ContentItemPropertyView;
use anyhow::anyhow;
use rs_content::TypedContent;
use rs_engine::build_content_file_url;
use rs_engine::content::ibl::IBL;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_localization::t;
use std::collections::HashMap;
use std::path::PathBuf;

enum EEventType {
    UpdateIBL(Option<PathBuf>, Option<PathBuf>),
}

impl UIEvent for EEventType {}
impl UIContentPropertyEvent for EEventType {}

pub(super) struct IBLContentEditable {}

impl ContentEditable for IBLContentEditable {
    fn render_thumbnail(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        project_folder_path: &std::path::Path,
        thumbnail_cache: &mut crate::thumbnail_cache::ThumbnailCache,
        expected_thumbnail_render_szie: egui::Vec2,
        ui: &mut egui::Ui,
    ) {
        let content = content.borrow();
        if let Some(ibl) = content.downcast_ref::<IBL>() {
            if let Some(image_reference) = &ibl.image_reference {
                let path = project_folder_path.join(image_reference);
                match thumbnail_cache.get_image_file_uri(&path) {
                    Some(uri) => {
                        ui.add_sized(expected_thumbnail_render_szie, egui::Image::new(uri));
                    }
                    None => {
                        thumbnail_cache.load_image(&path);
                        ui.add_sized(
                            expected_thumbnail_render_szie,
                            egui::Spinner::new().size(expected_thumbnail_render_szie.x),
                        );
                    }
                }
            }
        }
    }

    fn render_detail(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        content_item_property_view: &mut ContentItemPropertyView,
        ui: &mut egui::Ui,
    ) -> Option<Box<dyn UIContentPropertyEvent>> {
        let ibl_content = TypedContent::<IBL>::new(content).ok()?;
        let mut ibl = ibl_content.borrow_mut();

        let ibl_bake_info = &mut ibl.bake_info;
        ui.add(
            egui::DragValue::new(&mut ibl_bake_info.brdf_sample_count)
                .speed(1)
                .prefix(t!("BRDF Sample Count: "))
                .range(1..=8192),
        );
        ui.add(
            egui::DragValue::new(&mut ibl_bake_info.irradiance_sample_count)
                .speed(1)
                .prefix(t!("Irradiance Sample Count: "))
                .range(1..=8192),
        );
        ui.add(
            egui::DragValue::new(&mut ibl_bake_info.pre_filter_sample_count)
                .speed(1)
                .prefix(t!("Prefilter Sample Count: "))
                .range(1..=8192),
        );
        ui.add(
            egui::DragValue::new(&mut ibl_bake_info.brdflutmap_length)
                .speed(1)
                .prefix(t!("BRDF Length: "))
                .range(64..=2048),
        );
        ui.add(
            egui::DragValue::new(&mut ibl_bake_info.pre_filter_cube_map_max_mipmap_level)
                .speed(1)
                .prefix(t!("Prefilter Max Mipmap: "))
                .range(1..=64),
        );
        ui.add(
            egui::DragValue::new(&mut ibl_bake_info.irradiance_cube_map_length)
                .speed(1)
                .prefix(t!("Irradiance Length: "))
                .range(4..=8192),
        );
        ui.add(
            egui::DragValue::new(&mut ibl_bake_info.pre_filter_cube_map_length)
                .speed(1)
                .prefix(t!("Prefilter Cube Map Length: "))
                .range(4..=8192),
        );

        let old = ibl.image_reference.clone();
        let selected_text = if let Some(image_reference) = &ibl.image_reference {
            image_reference.to_str().unwrap().to_string()
        } else {
            t!("None").to_string()
        };

        let mut event: Option<EEventType> = None;
        egui::ComboBox::from_label("Asset")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut ibl.image_reference, None, t!("None"))
                    .clicked()
                {
                    event = Some(EEventType::UpdateIBL(old.clone(), None));
                }

                for image_asset_file in content_item_property_view.image_asset_files.iter() {
                    if ui
                        .selectable_value(
                            &mut ibl.image_reference,
                            Some(image_asset_file.clone()),
                            image_asset_file.to_str().unwrap(),
                        )
                        .clicked()
                    {
                        event = Some(EEventType::UpdateIBL(
                            old.clone(),
                            Some(image_asset_file.clone()),
                        ));
                    }
                }
            });

        if old != ibl.image_reference {
            event = Some(EEventType::UpdateIBL(old, ibl.image_reference.clone()));
        }

        event.map(|event| Box::new(event) as Box<dyn UIContentPropertyEvent>)
    }

    fn on_process_event(
        &self,
        editor_context: &mut crate::editor_context::EditorContext,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        event: Box<dyn UIContentPropertyEvent>,
    ) {
        let Some(event) = event.downcast_ref::<EEventType>() else {
            return;
        };
        match event {
            EEventType::UpdateIBL(old, new) => {
                let Some(new) = new.as_ref() else {
                    return;
                };
                let result: anyhow::Result<()> = (|| {
                    let url = {
                        let content_guard = content.borrow();
                        let ibl = content_guard
                            .downcast_ref::<IBL>()
                            .ok_or(anyhow::anyhow!(""))?;
                        ibl.url.clone()
                    };

                    let file_path = {
                        let project_context = editor_context
                            .project_context()
                            .ok_or(anyhow::anyhow!(""))?;
                        project_context.get_project_folder_path().join(new)
                    };
                    if !file_path.exists() {
                        return Err(anyhow::anyhow!("The file is not exist"));
                    }
                    let is_contains = editor_context
                        .engine_mut()
                        .get_resource_manager()
                        .get_ibl_textures()
                        .contains_key(&url);
                    if !is_contains {
                        let ibl_rc = content.clone();
                        crate::editor_context::EditorContext::load_ibl_content_resource(
                            editor_context,
                            ibl_rc,
                        )?;
                    }
                    Ok(())
                })();
                match result {
                    Ok(_) => {}
                    Err(err) => {
                        log::warn!("{}", err);
                        let mut content_guard = content.borrow_mut();
                        if let Some(ibl) = content_guard.downcast_mut::<IBL>() {
                            ibl.image_reference = old.clone();
                        }
                    }
                }
            }
        }
    }

    fn load_async<'a>(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        loading_context: PreLoadingContext<'a>,
        scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>>,
        engine: &mut rs_engine::engine::Engine,
    ) -> Vec<Box<dyn crate::load_content::types::PostLoading<Output = ()> + 'a>> {
        let _ = engine;
        let _ = scenes;
        let ibl = TypedContent::<IBL>::new(content).expect("Matched type");
        if let Some(future) =
            crate::load_content::load_ibl::LoadIBL::new(loading_context.clone(), ibl)
        {
            vec![Box::new(future)]
        } else {
            Vec::new()
        }
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn rs_artifact_types::asset::Asset>>,
        model_loader: &mut rs_model_loader::model_loader::ModelLoader,
        project_context: &crate::project_context::ProjectContext,
    ) -> anyhow::Result<()> {
        let _ = artifact_asset_encoder;
        let _ = model_loader;
        let ibl = TypedContent::<IBL>::new(content).expect("Matched type");
        let ibl = ibl.borrow();
        let project_folder_path = project_context.get_project_folder_path();

        let url = ibl.url.clone();
        let image_reference = &ibl.image_reference;
        let Some(image_reference) = image_reference.as_ref() else {
            return Err(anyhow!("The file is not exist"));
        };
        let file_path = project_folder_path.join(image_reference);
        if !file_path.exists() {
            return Err(anyhow!("The file is not exist"));
        }
        if !project_context
            .get_ibl_bake_cache_dir(image_reference)
            .exists()
        {
            return Err(anyhow!("The file is not exist"));
        }
        let name = rs_engine::url_extension::UrlExtension::get_name_in_editor(&url);
        let ibl_baking = rs_artifact::ibl_baking::IBLBaking {
            name,
            url: url.clone(),
            brdf_data: std::fs::read(
                project_context
                    .get_ibl_bake_cache_dir(image_reference)
                    .join("brdf.dds"),
            )?,
            pre_filter_data: std::fs::read(
                project_context
                    .get_ibl_bake_cache_dir(image_reference)
                    .join("pre_filter.dds"),
            )?,
            irradiance_data: std::fs::read(
                project_context
                    .get_ibl_bake_cache_dir(image_reference)
                    .join("irradiance.dds"),
            )?,
        };

        associated_assets.insert(ibl_baking.url.clone(), Box::new(ibl_baking));
        Ok(())
    }

    fn create_default(
        &self,
        name: String,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let _ = editor_context;
        let new_ibl = rs_engine::content::ibl::IBL::new(build_content_file_url(&name).unwrap());
        Some(Box::new(new_ibl))
    }

    fn display_name_for_creation(&self) -> Option<std::borrow::Cow<'static, str>> {
        Some(t!("IBL"))
    }
}
