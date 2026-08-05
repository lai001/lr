use crate::content_edit::{ContentEditable, UIContentPropertyEvent, UIEvent};
use crate::load_content::types::{PreLoadingContext, SceneWrapper};
use crate::ui::content_item_property_view::ContentItemPropertyView;
use anyhow::Context;
use rs_content::TypedContent;
use rs_engine::content::texture::TextureFile;
use rs_engine::file_type::EFileType;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_localization::t;
use std::{collections::HashMap, path::PathBuf};

enum EEventType {
    IsVirtualTexture(bool),
    SDF2D,
    IsCompressed(bool),
}

impl UIEvent for EEventType {}
impl UIContentPropertyEvent for EEventType {}

pub(super) struct TextureContentEditable {}

impl ContentEditable for TextureContentEditable {
    fn render_thumbnail(
        &self,
        content: rs_foundation::new::SingleThreadMutType<Box<dyn rs_content::Content>>,
        project_folder_path: &std::path::Path,
        thumbnail_cache: &mut crate::thumbnail_cache::ThumbnailCache,
        expected_thumbnail_render_szie: egui::Vec2,
        ui: &mut egui::Ui,
    ) {
        let content = content.borrow();
        if let Some(texture) = content.downcast_ref::<TextureFile>() {
            if let Some(image_reference) = texture.get_image_reference_path().as_ref() {
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
        content: rs_foundation::new::SingleThreadMutType<Box<dyn rs_content::Content>>,
        content_item_property_view: &mut ContentItemPropertyView,
        ui: &mut egui::Ui,
    ) -> Option<Box<dyn UIContentPropertyEvent>> {
        let _ = content_item_property_view;
        let texture_content = TypedContent::<TextureFile>::new(content).ok()?;
        let mut texture_file = texture_content.borrow_mut();

        if ui
            .checkbox(
                &mut texture_file.is_virtual_texture,
                t!("Is Virtual Texture"),
            )
            .changed()
        {
            return Some(Box::new(EEventType::IsVirtualTexture(
                texture_file.is_virtual_texture,
            )));
        }

        if ui.button("SDF 2D").clicked() {
            return Some(Box::new(EEventType::SDF2D));
        }

        let mut is_compressed = texture_file.is_compressed;
        if ui
            .checkbox(&mut is_compressed, t!("Is compressed"))
            .changed()
        {
            return Some(Box::new(EEventType::IsCompressed(is_compressed)));
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
            EEventType::IsVirtualTexture(is_virtual_texture) => {
                let result: anyhow::Result<()> = (|| {
                    if !is_virtual_texture {
                        return Ok(());
                    }
                    let project_context = editor_context
                        .project_context()
                        .ok_or(anyhow::anyhow!("No project context"))?;
                    let virtual_texture_cache_dir =
                        project_context.try_create_virtual_texture_cache_dir()?;
                    let project_folder_path = &project_context.get_project_folder_path();

                    let virtual_cache_name = {
                        let content_guard = content.borrow();
                        let texture = content_guard
                            .downcast_ref::<TextureFile>()
                            .ok_or(anyhow::anyhow!(""))?;
                        texture.get_pref_virtual_cache_name(project_folder_path)?
                    };
                    {
                        let mut content_guard = content.borrow_mut();
                        let texture = content_guard
                            .downcast_mut::<TextureFile>()
                            .ok_or(anyhow::anyhow!(""))?;
                        texture.create_virtual_texture_cache(
                            project_folder_path,
                            &virtual_texture_cache_dir.join(virtual_cache_name.clone()),
                            Some(rs_artifact::EEndianType::Little),
                            256,
                        )?;
                        log::trace!("virtual_cache_name: {}", virtual_cache_name);
                        texture.virtual_image_reference = Some(virtual_cache_name);
                    }
                    Ok(())
                })();
                log::trace!("{:?}", result);
            }
            EEventType::SDF2D => {
                let result: anyhow::Result<()> = (|| {
                    let content_guard = content.borrow();
                    let texture = content_guard
                        .downcast_ref::<TextureFile>()
                        .ok_or(anyhow::anyhow!(""))?;
                    let image_reference = texture
                        .image_reference
                        .as_ref()
                        .ok_or(anyhow::anyhow!(""))?;
                    let project_context = editor_context
                        .project_context()
                        .ok_or(anyhow::anyhow!(""))?;
                    let path = project_context.get_asset_path_by_url(image_reference);
                    let image = image::open(path)?;
                    let image = image.to_rgba8();
                    editor_context.engine_mut().sdf2d(image);
                    Ok(())
                })();
                log::trace!("{:?}", result);
            }
            EEventType::IsCompressed(is_compressed) => {
                if let Some(project_context) = editor_context.project_context() {
                    if *is_compressed {
                        let mut content_guard = content.borrow_mut();
                        let Some(texture) = content_guard.downcast_mut::<TextureFile>() else {
                            return;
                        };
                        match crate::editor_context::EditorContext::create_compressed_texture(
                            project_context,
                            &texture,
                        ) {
                            Ok(compressed_texture) => {
                                if let Ok(dir) = project_context.try_create_derive_data_dir() {
                                    let apply_result = editor_context.apply_compressed_texture(
                                        texture,
                                        compressed_texture,
                                        &dir,
                                    );
                                    if let Err(err) = apply_result {
                                        log::warn!("{}", err);
                                    }
                                }
                            }
                            Err(err) => log::warn!("{}", err),
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
        let _ = scenes;
        let _ = engine;
        let texture = TypedContent::<TextureFile>::new(content).expect("Matched type");
        let mut futures = Vec::new();
        if let Some(future) = crate::load_content::load_texture::LoadTexture::new(
            loading_context.clone(),
            texture.clone(),
        ) {
            futures.push(Box::new(future)
                as Box<dyn crate::load_content::types::PostLoading<Output = ()> + 'a>);
        }
        if let Some(future) = crate::load_content::load_virtual_texture::LoadVirtualTexture::new(
            loading_context.clone(),
            texture,
        ) {
            futures.push(Box::new(future)
                as Box<dyn crate::load_content::types::PostLoading<Output = ()> + 'a>);
        }
        futures
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn rs_artifact::asset::Asset>>,
        model_loader: &mut rs_model_loader::model_loader::ModelLoader,
        project_context: &crate::project_context::ProjectContext,
    ) -> anyhow::Result<()> {
        let _ = model_loader;
        let texture = TypedContent::<TextureFile>::new(content).expect("Matched type");
        let texture = texture.borrow();
        if let Some(image_reference) = &texture.image_reference {
            let absolute_image_file_path = project_context.get_asset_path_by_url(image_reference);

            let buffer = std::fs::read(absolute_image_file_path.clone()).context(format!(
                "Failed to read from {:?}",
                absolute_image_file_path
            ))?;
            let _ = image::load_from_memory(&buffer).context(format!(
                "{:?} is not a valid image file.",
                absolute_image_file_path
            ))?;
            let format = image::guess_format(&buffer)?;
            let image = rs_artifact::image::Image {
                url: image_reference.clone(),
                image_format: rs_artifact::image::ImageFormat::from_external_format(format),
                data: buffer,
            };
            associated_assets.insert(image_reference.clone(), Box::new(image));
        }
        artifact_asset_encoder.encode(&*texture);

        Ok(())
    }

    fn is_support_asset_file(&self, asset_file: &crate::data_source::AssetFile) -> bool {
        match asset_file.get_file_type() {
            EFileType::Jpeg | EFileType::Jpg | EFileType::Png | EFileType::Exr | EFileType::Hdr => {
                true
            }
            _ => false,
        }
    }

    fn display_name_for_creation(&self) -> Option<std::borrow::Cow<'static, str>> {
        Some(t!("Texture"))
    }

    fn create_from_asset_file(
        &self,
        name: String,
        asset_file: &crate::data_source::AssetFile,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let current_folder = editor_context
            .data_source_mut()
            .content_data_source
            .current_folder
            .clone();

        if let Some(project_context) = editor_context.project_context_mut() {
            let asset_folder_path = project_context.get_asset_folder_path();
            let image_reference: PathBuf = {
                if asset_file.path.starts_with(asset_folder_path.clone()) {
                    asset_file
                        .path
                        .strip_prefix(asset_folder_path)
                        .unwrap()
                        .to_path_buf()
                } else {
                    asset_file.path.clone()
                }
            };
            if let Some(current_folder) = &current_folder {
                let folder_url = current_folder.get_url();
                let url = folder_url.join(&name).unwrap();
                let mut texture_file = TextureFile::new(url);
                texture_file.set_image_reference_path(image_reference);
                log::trace!("Create texture: {:?}", &texture_file.url.as_str());
                let content_manager = project_context.content_manager.clone();
                let content_manager = content_manager.borrow_mut();
                editor_context
                    .data_source_mut()
                    .content_data_source
                    .current_folder = content_manager
                    .content_folders()
                    .get(current_folder.relative_path())
                    .cloned();
                return Some(Box::new(texture_file));
            }
        }

        return None;
    }
}
