use std::collections::HashMap;

use crate::{content_edit::ContentEditable, project_context::ProjectContext};
use rs_artifact_types::asset::Asset;
use rs_content::TypedContent;
use rs_engine::content::level::Level;
use rs_foundation::new::SingleThreadMutType;
use rs_model_loader::model_loader::ModelLoader;
use rust_i18n::t;

pub(super) struct LevelContentEditable {}

impl ContentEditable for LevelContentEditable {
    fn render_thumbnail(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        project_folder_path: &std::path::Path,
        thumbnail_cache: &mut crate::thumbnail_cache::ThumbnailCache,
        expected_thumbnail_render_szie: egui::Vec2,
        ui: &mut egui::Ui,
    ) {
        let _ = content;
        let _ = project_folder_path;
        let _ = thumbnail_cache;
        let _ = expected_thumbnail_render_szie;
        ui.image(egui::include_image!("../../../Resource/Editor/level.svg"));
    }

    fn open(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        editor_context: &mut crate::editor_context::EditorContext,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let _ = event_loop_window_target;
        let level = TypedContent::<Level>::new(content).expect("Matched type");
        let project_context = editor_context.project_context();
        let Some(project_context) = project_context.as_ref() else {
            return;
        };
        let content_manager = project_context.content_manager.clone();
        let content_manager = content_manager.borrow();
        editor_context.open_level(level, &content_manager);
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn Asset>>,
        model_loader: &mut ModelLoader,
        project_context: &ProjectContext,
    ) -> anyhow::Result<()> {
        let _ = associated_assets;
        let _ = project_context;
        let _ = model_loader;
        let level = TypedContent::<Level>::new(content).expect("Matched type");
        let level = level.borrow();
        artifact_asset_encoder.encode_content(&*level);
        Ok(())
    }

    fn create_default(
        &self,
        name: String,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let _ = editor_context;
        let level = Level::new(name);
        Some(Box::new(level))
    }

    fn display_name_for_creation(&self) -> Option<std::borrow::Cow<'static, str>> {
        Some(t!("Level"))
    }
}
