mod blend_animations;
mod curve;
mod ibl;
mod level;
mod material;
mod material_paramenters_collection;
mod particle_system;
mod render_target_2d;
mod skeleton;
mod skeleton_animation;
mod skeleton_mesh;
mod sound;
mod static_mesh;
mod texture;

use crate::{
    data_source::AssetFile,
    load_content::types::{PreLoadingContext, SceneWrapper},
    project_context::ProjectContext,
    ui::{UIEvent, content_item_property_view::ContentItemPropertyView},
};
use anyhow::anyhow;
use downcast_rs::impl_downcast;
use rs_artifact::asset::Asset;
use rs_content::Content;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_model_loader::model_loader::ModelLoader;
use std::{any::TypeId, borrow::Cow, collections::HashMap, path::PathBuf};

pub trait UIContentPropertyEvent: UIEvent {}
impl_downcast!(UIContentPropertyEvent);

pub trait ContentEditable: 'static {
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
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 5.0, egui::Color32::WHITE);
    }

    fn render_detail(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        content_item_property_view: &mut ContentItemPropertyView,
        ui: &mut egui::Ui,
    ) -> Option<Box<dyn UIContentPropertyEvent>> {
        let _ = content_item_property_view;
        let _ = ui;
        let _ = content;
        None
    }

    fn on_process_event(
        &self,
        editor_context: &mut crate::editor_context::EditorContext,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        event: Box<dyn UIContentPropertyEvent>,
    ) {
        let _ = editor_context;
        let _ = content;
        let _ = event;
    }

    fn open(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        editor_context: &mut crate::editor_context::EditorContext,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
    ) {
        let _ = event_loop_window_target;
        let _ = editor_context;
        let _ = content;
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn Asset>>,
        model_loader: &mut ModelLoader,
        project_context: &ProjectContext,
    ) -> anyhow::Result<()> {
        let _ = project_context;
        let _ = associated_assets;
        let _ = model_loader;
        let _ = artifact_asset_encoder;
        let _ = content;
        Err(anyhow!("not implemented"))
    }

    fn display_type_name(&self, content: &dyn rs_content::Content) -> Cow<'static, str> {
        std::borrow::Cow::Borrowed(content.get_type_text())
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
        let _ = loading_context;
        let _ = content;
        Vec::new()
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
        let _ = engine;
        let _ = content;
    }

    fn create_default(
        &self,
        name: String,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let _ = editor_context;
        let _ = name;
        None
    }

    fn is_support_asset_file(&self, asset_file: &AssetFile) -> bool {
        let _ = asset_file;
        false
    }

    fn create_from_asset_file(
        &self,
        name: String,
        asset_file: &crate::data_source::AssetFile,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let _ = asset_file;
        let _ = editor_context;
        let _ = name;
        None
    }

    fn display_name_for_creation(&self) -> Option<Cow<'static, str>> {
        None
    }
}

pub struct ContentEdit {
    editables: HashMap<TypeId, Box<dyn ContentEditable>>,
}

impl ContentEdit {
    pub fn new() -> ContentEdit {
        let mut editables: HashMap<TypeId, Box<dyn ContentEditable>> = HashMap::new();
        editables.insert(
            TypeId::of::<rs_engine::content::static_mesh::StaticMesh>(),
            Box::new(static_mesh::StaticMeshContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::skeleton_mesh::SkeletonMesh>(),
            Box::new(skeleton_mesh::SkeletonMeshContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::skeleton_animation::SkeletonAnimation>(),
            Box::new(skeleton_animation::SkeletonAnimationContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::skeleton::Skeleton>(),
            Box::new(skeleton::SkeletonContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::texture::TextureFile>(),
            Box::new(texture::TextureContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::level::Level>(),
            Box::new(level::LevelContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::material::Material>(),
            Box::new(material::MaterialContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::ibl::IBL>(),
            Box::new(ibl::IBLContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::particle_system::ParticleSystem>(),
            Box::new(particle_system::ParticleSystemContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::sound::Sound>(),
            Box::new(sound::SoundContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::curve::Curve>(),
            Box::new(curve::CurveContentEditable {}),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::blend_animations::BlendAnimations>(),
            Box::new(blend_animations::BlendAnimationsContentEditable {}),
        );
        editables.insert(
            TypeId::of::<
                rs_engine::content::material_paramenters_collection::MaterialParamentersCollection,
            >(),
            Box::new(
                material_paramenters_collection::MaterialParamentersCollectionContentEditable {},
            ),
        );
        editables.insert(
            TypeId::of::<rs_engine::content::render_target_2d::RenderTarget2D>(),
            Box::new(render_target_2d::RenderTarget2DContentEditable {}),
        );
        ContentEdit { editables }
    }

    pub fn editable(&mut self, content: &dyn Content) -> Option<&mut Box<dyn ContentEditable>> {
        let id = content.type_id();
        let editable = self.editables.get_mut(&id);
        editable
    }

    pub fn editable_id(&mut self, type_id: &TypeId) -> Option<&mut Box<dyn ContentEditable>> {
        let editable = self.editables.get_mut(type_id);
        editable
    }

    pub fn editables(&self) -> &HashMap<TypeId, Box<dyn ContentEditable>> {
        &self.editables
    }
}
