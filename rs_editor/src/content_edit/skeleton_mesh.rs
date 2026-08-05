use crate::content_edit::ContentEditable;
use crate::load_content::types::{PreLoadingContext, SceneWrapper};
use rs_content::TypedContent;
use rs_engine::content::skeleton_mesh::SkeletonMesh;
use rs_engine::resource_manager::ResourceManager;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use std::ops::Deref;
use std::{collections::HashMap, path::PathBuf};

pub(super) struct SkeletonMeshContentEditable {}

impl ContentEditable for SkeletonMeshContentEditable {
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
            "../../../Resource/Editor/skeleton_mesh.svg"
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
        let skeleton_mesh = TypedContent::<SkeletonMesh>::new(content).expect("Matched type");
        if let Some(future) = crate::load_content::load_skeleton_mesh::LoadSkeletonMesh::new(
            loading_context.clone(),
            skeleton_mesh,
            scenes.clone(),
        ) {
            vec![Box::new(future)]
        } else {
            Vec::new()
        }
    }

    fn open(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        editor_context: &mut crate::editor_context::EditorContext,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let skeleton_mesh = TypedContent::<SkeletonMesh>::new(content).expect("Matched type");
        editor_context
            .open_skin_mesh_window(&mut skeleton_mesh.borrow_mut(), event_loop_window_target);
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn rs_artifact::asset::Asset>>,
        model_loader: &mut rs_model_loader::model_loader::ModelLoader,
        project_context: &crate::project_context::ProjectContext,
    ) -> anyhow::Result<()> {
        let skeleton_mesh = TypedContent::<SkeletonMesh>::new(content).expect("Matched type");
        let project_folder_path = project_context.get_project_folder_path();
        let skeleton_mesh = skeleton_mesh.borrow();

        let file_path = project_folder_path.join(&skeleton_mesh.get_relative_path());
        model_loader
            .load_scene_from_file_and_cache(&file_path)
            .unwrap();
        let loaded_skin_mesh = model_loader.to_runtime_cache_skin_mesh(
            &skeleton_mesh,
            &project_folder_path,
            ResourceManager::default(),
        );
        associated_assets.insert(
            loaded_skin_mesh.url.clone(),
            Box::new(loaded_skin_mesh.deref().clone()),
        );

        artifact_asset_encoder.encode(&*skeleton_mesh);

        Ok(())
    }
}
