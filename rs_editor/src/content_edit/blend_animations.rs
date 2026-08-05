use crate::content_edit::ContentEditable;
use rs_content::TypedContent;
use rs_engine::{build_content_file_url, content::blend_animations::BlendAnimations};
use rs_foundation::new::SingleThreadMutType;
use rust_i18n::t;
use std::collections::HashMap;

pub(super) struct BlendAnimationsContentEditable {}

impl ContentEditable for BlendAnimationsContentEditable {
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
        ui.image(egui::include_image!(
            "../../../Resource/Editor/blend_animations.svg"
        ));
    }

    fn open(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        editor_context: &mut crate::editor_context::EditorContext,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let blend_animations = TypedContent::<BlendAnimations>::new(content).expect("Matched type");
        editor_context.open_blend_animation_ui_window(blend_animations, event_loop_window_target);
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn rs_artifact::asset::Asset>>,
        model_loader: &mut rs_model_loader::model_loader::ModelLoader,
        project_context: &crate::project_context::ProjectContext,
    ) -> anyhow::Result<()> {
        let _ = project_context;
        let _ = model_loader;
        let _ = associated_assets;
        let blend_animations = TypedContent::<BlendAnimations>::new(content).expect("Matched type");
        let blend_animations = blend_animations.borrow();
        artifact_asset_encoder.encode(&*blend_animations);
        Ok(())
    }

    fn create_default(
        &self,
        name: String,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let _ = editor_context;
        let content_url = build_content_file_url(&name).ok()?;
        let blend_animation =
            rs_engine::content::blend_animations::BlendAnimations::new(content_url);
        Some(Box::new(blend_animation))
    }

    fn display_name_for_creation(&self) -> Option<std::borrow::Cow<'static, str>> {
        Some(t!("Blend Animation"))
    }
}
