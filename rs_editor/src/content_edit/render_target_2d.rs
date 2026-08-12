use crate::content_edit::{ContentEditable, UIContentPropertyEvent, UIEvent};
use crate::ui::content_item_property_view::ContentItemPropertyView;
use crate::ui::misc::render_combo_box_not_null;
use rs_content::TypedContent;
use rs_engine::build_content_file_url;
use rs_engine::content::render_target_2d::RenderTarget2D;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_localization::t;
use std::collections::HashMap;
use std::path::PathBuf;

enum EEventType {
    Update,
}

impl UIEvent for EEventType {}
impl UIContentPropertyEvent for EEventType {}

pub(super) struct RenderTarget2DContentEditable {}

impl ContentEditable for RenderTarget2DContentEditable {
    fn render_detail(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        content_item_property_view: &mut ContentItemPropertyView,
        ui: &mut egui::Ui,
    ) -> Option<Box<dyn UIContentPropertyEvent>> {
        let _ = content_item_property_view;
        let rt_content = TypedContent::<RenderTarget2D>::new(content).ok()?;
        let mut render_target_2d = rt_content.borrow_mut();

        let response = ui.add(
            egui::DragValue::new(&mut render_target_2d.width)
                .speed(1)
                .prefix(t!("Width: "))
                .range(1..=4096 * 4)
                .update_while_editing(false),
        );
        if response.lost_focus() {
            return Some(Box::new(EEventType::Update));
        }

        let response = ui.add(
            egui::DragValue::new(&mut render_target_2d.height)
                .speed(1)
                .prefix(t!("Height: "))
                .range(1..=4096 * 4)
                .update_while_editing(false),
        );
        if response.lost_focus() {
            return Some(Box::new(EEventType::Update));
        }

        let candidate_items = vec![
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::R8Unorm,
        ];
        if render_combo_box_not_null(
            ui,
            t!("Texture Format"),
            "TextureFormat",
            &mut render_target_2d.format,
            candidate_items,
        ) {
            return Some(Box::new(EEventType::Update));
        }

        None
    }

    fn on_process_event(
        &self,
        editor_context: &mut crate::editor_context::EditorContext,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        event: Box<dyn UIContentPropertyEvent>,
    ) {
        let Some(_event) = event.downcast_ref::<EEventType>() else {
            return;
        };
        let mut content_guard = content.borrow_mut();
        if let Some(render_target_2d) = content_guard.downcast_mut::<RenderTarget2D>() {
            render_target_2d.init_resouce(editor_context.engine_mut());
        }
    }

    fn load_sync<'a>(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        loading_context: crate::load_content::types::PreLoadingContext<'a>,
        scenes: MultipleThreadMutType<HashMap<PathBuf, crate::load_content::types::SceneWrapper>>,
        engine: &mut rs_engine::engine::Engine,
    ) {
        let _ = scenes;
        let _ = loading_context;
        let render_target_2d = TypedContent::<RenderTarget2D>::new(content).expect("Matched type");
        let mut render_target_2d = render_target_2d.borrow_mut();
        render_target_2d.init_resouce(engine);
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn rs_artifact_types::asset::Asset>>,
        model_loader: &mut rs_model_loader::model_loader::ModelLoader,
        project_context: &crate::project_context::ProjectContext,
    ) -> anyhow::Result<()> {
        let _ = project_context;
        let _ = model_loader;
        let _ = associated_assets;
        let render_target_2d = TypedContent::<RenderTarget2D>::new(content).expect("Matched type");
        let render_target_2d = render_target_2d.borrow();
        artifact_asset_encoder.encode_content(&*render_target_2d);
        Ok(())
    }

    fn create_default(
        &self,
        name: String,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let content_url = build_content_file_url(&name).ok()?;
        let length = RenderTarget2D::default_length();
        let mut render_target_2d = RenderTarget2D::new(content_url, length, length, None);
        render_target_2d.init_resouce(editor_context.engine_mut());
        Some(Box::new(render_target_2d))
    }

    fn display_name_for_creation(&self) -> Option<std::borrow::Cow<'static, str>> {
        Some(t!("RenderTarget2D"))
    }
}
