use crate::content_edit::ContentEditable;
use rs_content::TypedContent;
use rs_engine::{build_content_file_url, content::particle_system::ParticleSystem};
use rs_foundation::new::SingleThreadMutType;
use rs_model_loader::model_loader::ModelLoader;
use rust_i18n::t;
use std::collections::HashMap;

pub(super) struct ParticleSystemContentEditable {}

impl ContentEditable for ParticleSystemContentEditable {
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
            "../../../Resource/Editor/particle.svg"
        ));
    }

    fn open(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        editor_context: &mut crate::editor_context::EditorContext,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let particle_system = TypedContent::<ParticleSystem>::new(content).expect("Matched type");
        editor_context.open_particle_window(event_loop_window_target, particle_system);
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn rs_artifact_types::asset::Asset>>,
        model_loader: &mut ModelLoader,
        project_context: &crate::project_context::ProjectContext,
    ) -> anyhow::Result<()> {
        let _ = project_context;
        let _ = model_loader;
        let _ = associated_assets;
        let particle_system = TypedContent::<ParticleSystem>::new(content).expect("Matched type");
        let particle_system = particle_system.borrow();
        artifact_asset_encoder.encode_content(&*particle_system);
        Ok(())
    }

    fn create_default(
        &self,
        name: String,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let _ = editor_context;
        let particle_system = rs_engine::content::particle_system::ParticleSystem::new(
            build_content_file_url(&name).unwrap(),
        );
        Some(Box::new(particle_system))
    }

    fn display_name_for_creation(&self) -> Option<std::borrow::Cow<'static, str>> {
        Some(t!("Particle System"))
    }
}
