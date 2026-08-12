use crate::content_edit::ContentEditable;
use crate::load_content::types::{PreLoadingContext, SceneWrapper};
use rs_content::TypedContent;
use rs_engine::content::skeleton::Skeleton;
use rs_engine::resource_manager::ResourceManager;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use std::ops::Deref;
use std::{collections::HashMap, path::PathBuf};

pub(super) struct SkeletonContentEditable {}

impl ContentEditable for SkeletonContentEditable {
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
        ui.image(egui::include_image!(
            "../../../Resource/Editor/skeleton.svg"
        ));
    }

    fn load_async<'a>(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        loading_context: PreLoadingContext<'a>,
        scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>>,
        engine: &mut rs_engine::engine::Engine,
    ) -> Vec<Box<dyn crate::load_content::types::PostLoading<Output = ()> + 'a>> {
        let _ = engine;
        let skeleton = TypedContent::<Skeleton>::new(content).expect("Matched type");
        if let Some(future) = crate::load_content::load_skeleton::LoadSkeleton::new(
            loading_context.clone(),
            skeleton,
            scenes.clone(),
        ) {
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
        let skeleton = TypedContent::<Skeleton>::new(content).expect("Matched type");
        let skeleton = skeleton.borrow();
        let project_folder_path = project_context.get_project_folder_path();

        let file_path = project_folder_path.join(&skeleton.get_relative_path());
        model_loader
            .load_scene_from_file_and_cache(&file_path)
            .unwrap();
        let loaded_skeleton = model_loader.to_runtime_cache_skeleton(
            &skeleton,
            &project_folder_path,
            ResourceManager::default(),
        );
        associated_assets.insert(
            loaded_skeleton.url.clone(),
            Box::new(loaded_skeleton.deref().clone()),
        );

        artifact_asset_encoder.encode_content(&*skeleton);

        Ok(())
    }
}
