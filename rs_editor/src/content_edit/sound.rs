use crate::content_edit::ContentEditable;
use crate::load_content::types::{PreLoadingContext, SceneWrapper};
use rs_artifact::sound::ESoundFileType;
use rs_content::TypedContent;
use rs_engine::content::sound::Sound;
use rs_engine::file_type::EFileType;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use std::{collections::HashMap, path::PathBuf};

pub(super) struct SoundContentEditable {}

impl ContentEditable for SoundContentEditable {
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
        ui.image(egui::include_image!("../../../Resource/Editor/sound.svg"));
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
        let sound = TypedContent::<Sound>::new(content).expect("Matched type");
        if let Some(future) =
            crate::load_content::load_sound::LoadSound::new(loading_context.clone(), sound)
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
        let _ = model_loader;
        let sound = TypedContent::<Sound>::new(content).expect("Matched type");
        let sound = sound.borrow();
        artifact_asset_encoder.encode_content(&*sound);
        let path = project_context
            .get_asset_folder_path()
            .join(&sound.asset_info.relative_path);
        let data = std::fs::read(path)?;
        let sound_resource = rs_artifact::sound::Sound {
            url: sound.asset_info.get_url(),
            sound_file_type: ESoundFileType::Unknow,
            data,
        };
        associated_assets.insert(sound.asset_info.get_url(), Box::new(sound_resource));
        Ok(())
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

        let Some(current_folder) = &current_folder else {
            return None;
        };
        let Some(project_context) = editor_context.project_context_mut() else {
            return None;
        };
        let asset_folder_path = project_context.get_asset_folder_path();

        let relative_path: PathBuf = {
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

        let folder_url = current_folder.get_url();
        let url = folder_url.join(&name).unwrap();

        let sound = rs_engine::content::sound::Sound::new(url, relative_path);
        let content = Box::new(sound);
        let content_manager = project_context.content_manager.clone();
        let content_manager = content_manager.borrow_mut();
        editor_context
            .data_source_mut()
            .content_data_source
            .current_folder = content_manager
            .content_folders()
            .get(current_folder.relative_path())
            .cloned();
        return Some(content);
    }

    fn is_support_asset_file(&self, asset_file: &crate::data_source::AssetFile) -> bool {
        match asset_file.get_file_type() {
            EFileType::WAV | EFileType::MP3 => true,
            _ => false,
        }
    }
}
