use crate::content_edit::ContentEditable;
use crate::load_content::types::{PreLoadingContext, SceneWrapper};
use crate::ui::content_item_property_view::ContentItemPropertyView;
use crate::ui::material_view::{self, EMaterialNodeType, MaterialNode};
use crate::ui::misc::render_combo_box_not_null;
use anyhow::anyhow;
use rs_artifact::material::MaterialInfo;
use rs_content::TypedContent;
use rs_engine::build_content_file_url;
use rs_engine::content::material::Material;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMut, SingleThreadMutType};
use rs_localization::t;
use rs_render_types::{EBlendModeType, MaterialOptions};
use std::{collections::HashMap, path::PathBuf};

pub(super) struct MaterialContentEditable {}

impl ContentEditable for MaterialContentEditable {
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
            "../../../Resource/Editor/material.svg"
        ));
    }

    fn render_detail(
        &self,
        content: rs_foundation::new::SingleThreadMutType<Box<dyn rs_content::Content>>,
        content_item_property_view: &mut ContentItemPropertyView,
        ui: &mut egui::Ui,
    ) -> Option<Box<dyn super::UIContentPropertyEvent>> {
        let _ = content_item_property_view;
        let material_content = TypedContent::<Material>::new(content).ok()?;
        let mut material = material_content.borrow_mut();

        render_combo_box_not_null(
            ui,
            t!("Blend Mode"),
            "Blend Mode",
            &mut material.blend_mode,
            vec![EBlendModeType::Opaque, EBlendModeType::Transparent],
        );

        None
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
        let material = TypedContent::<Material>::new(content).expect("Matched type");
        if let Some(future) =
            crate::load_content::load_material::LoadMaterial::new(loading_context.clone(), material)
        {
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
        let material = TypedContent::<Material>::new(content).expect("Matched type");
        editor_context.open_material_window(event_loop_window_target, material);
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
        let material_content = TypedContent::<Material>::new(content).expect("Matched type");
        let material_content = material_content.borrow();

        let material_editor = project_context
            .project
            .materials
            .iter()
            .find(|x| x.borrow().url == material_content.asset_url)
            .cloned()
            .ok_or(anyhow!("Miss material"))?;
        let material_editor = material_editor.borrow();
        let snarl = &material_editor.snarl;
        let paramenters = &material_editor.paramenters;

        let resolve_result = crate::material_resolve::resolve(
            project_context.module_manager.clone(),
            project_context.content_manager.clone(),
            Some(&material_content.url),
            snarl,
            MaterialOptions::all(),
            paramenters,
        )?;

        let mut shader_code: HashMap<MaterialOptions, String> = HashMap::new();
        let mut material_info: HashMap<MaterialOptions, MaterialInfo> = HashMap::new();

        for (option, result) in resolve_result {
            shader_code.insert(option.clone(), result.shader_code);
            material_info.insert(option, result.material_info);
        }

        associated_assets.insert(
            material_content.asset_url.clone(),
            Box::new(rs_artifact::material::Material {
                url: material_content.asset_url.clone(),
                code: shader_code,
                material_info: material_info,
            }),
        );
        artifact_asset_encoder.encode(&*material_content);
        Ok(())
    }

    fn create_default(
        &self,
        name: String,
        editor_context: &mut crate::editor_context::EditorContext,
    ) -> Option<Box<dyn rs_content::Content>> {
        let module_manager = { editor_context.project_context_mut()? }
            .module_manager
            .clone();
        let content_manager = { editor_context.project_context_mut()? }
            .content_manager
            .clone();

        let content_url = build_content_file_url(&name).unwrap();
        let asset_url = crate::material::Material::make_url(&content_url);
        let mut material = rs_engine::content::material::Material::new(content_url, asset_url);
        let resolve_result =
            material_view::MaterialView::default_resolve(module_manager, content_manager).unwrap();
        {
            let mut shader_code = HashMap::new();
            let mut material_info = HashMap::new();
            for (k, v) in resolve_result.iter() {
                shader_code.insert(k.clone(), v.shader_code.clone());
                material_info.insert(k.clone(), v.material_info.clone());
            }
            let handle = editor_context.engine_mut().create_material(shader_code);
            material.set_pipeline_handle(handle);
            material.set_material_info(editor_context.engine_mut(), material_info);
        }
        let material_editor = crate::material::Material::new(material.asset_url.clone(), {
            let mut snarl = egui_snarl::Snarl::new();
            let node = MaterialNode {
                node_type: EMaterialNodeType::Sink(Default::default()),
            };
            snarl.insert_node(egui::pos2(0.0, 0.0), node);
            snarl
        });
        if editor_context
            .engine_mut()
            .get_settings()
            .render_setting
            .is_enable_dump_material_shader_code
        {
            if let Err(err) = crate::editor_context::EditorContext::write_debug_shader(
                &material_editor,
                &resolve_result,
            ) {
                log::warn!("{}", err);
            }
        }
        let Some(project_context) = editor_context.project_context_mut() else {
            return None;
        };
        project_context
            .project
            .materials
            .push(SingleThreadMut::new(material_editor));

        let mut materials = editor_context
            .editor_ui_mut()
            .object_property_view
            .materials
            .borrow_mut();
        if !materials.contains(&build_content_file_url(&name).unwrap()) {
            materials.push(build_content_file_url(&name).unwrap());
        }
        Some(Box::new(material))
    }

    fn display_name_for_creation(&self) -> Option<std::borrow::Cow<'static, str>> {
        Some(t!("Material"))
    }
}
