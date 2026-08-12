use crate::{
    content_edit::{ContentEditable, UIContentPropertyEvent, UIEvent},
    load_content::types::{PreLoadingContext, SceneWrapper},
    project_context::ProjectContext,
    ui::content_item_property_view::ContentItemPropertyView,
};
use rs_artifact_types::asset::Asset;
use rs_content::TypedContent;
use rs_engine::{content::static_mesh::StaticMesh, resource_manager::ResourceManager};
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_metis::{cluster::ClusterCollection, vertex_position::VertexPosition};
use rs_model_loader::model_loader::ModelLoader;
use rust_i18n::t;
use std::{collections::HashMap, ops::Deref, path::PathBuf, sync::Arc};

enum EEventType {
    UpdateStaticMeshEnableMultiresolution(bool),
}

impl UIEvent for EEventType {}
impl UIContentPropertyEvent for EEventType {}

pub(super) struct StaticMeshContentEditable {}

impl StaticMeshContentEditable {
    fn create_multi_res_mesh_cache_non_blocking(
        project_context: &crate::project_context::ProjectContext,
        static_mesh: &rs_engine::content::static_mesh::StaticMesh,
    ) -> anyhow::Result<()> {
        if !static_mesh.is_enable_multiresolution {
            return Ok(());
        }
        rs_core_minimal::thread_pool::ThreadPool::global().spawn({
            let mesh_cluster_dir = project_context.try_create_mesh_cluster_dir()?;
            let static_mesh_artiface_url = static_mesh.asset_info.get_url();
            move || match Self::create_multi_res_mesh_cache(
                &mesh_cluster_dir,
                static_mesh_artiface_url,
            ) {
                Ok(_) => {}
                Err(err) => {
                    log::warn!("{}", err);
                }
            }
        });
        Ok(())
    }

    fn create_multi_res_mesh_cache(
        mesh_cluster_dir: &std::path::Path,
        static_mesh_artiface_url: url::Url,
    ) -> anyhow::Result<rs_metis::cluster::ClusterCollection> {
        let rm = ResourceManager::default();
        let static_mesh_result = rm.get_static_mesh(&static_mesh_artiface_url)?;
        let indices = &static_mesh_result.indexes;

        let mut vertices: Vec<VertexPosition> =
            Vec::with_capacity(static_mesh_result.vertexes.len());
        for item in static_mesh_result.vertexes.iter() {
            vertices.push(VertexPosition::new(item.position));
        }
        let vertices = Arc::new(vertices);
        let gpmetis_program_path: Option<std::path::PathBuf> = None;

        let cluster_collection = ClusterCollection::parallel_from_indexed_vertices(
            indices,
            vertices,
            gpmetis_program_path,
        )?;

        let filename = static_mesh_result.name.clone();
        let output_path = mesh_cluster_dir.join(filename);
        let data = rs_artifact::bincode_legacy::serialize(&cluster_collection, None)?;
        let _ = std::fs::write(output_path, data)?;
        Ok(cluster_collection)
    }
}

impl ContentEditable for StaticMeshContentEditable {
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
            "../../../Resource/Editor/static_mesh.svg"
        ));
    }

    fn render_detail(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        content_item_property_view: &mut ContentItemPropertyView,
        ui: &mut egui::Ui,
    ) -> Option<Box<dyn super::UIContentPropertyEvent>> {
        let _ = content_item_property_view;
        let static_mesh = TypedContent::<StaticMesh>::new(content).ok()?;
        let static_mesh = static_mesh.borrow();
        let old_value = static_mesh.is_enable_multiresolution;
        let mut new_value = static_mesh.is_enable_multiresolution;
        ui.label(format!(
            "Asset url: {}",
            static_mesh.asset_info.get_url().to_string()
        ));
        ui.checkbox(&mut new_value, t!("Is enable multiresolution"));
        if old_value != new_value {
            return Some(Box::new(EEventType::UpdateStaticMeshEnableMultiresolution(
                new_value,
            )));
        }
        return None;
    }

    fn on_process_event(
        &self,
        editor_context: &mut crate::editor_context::EditorContext,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        event: Box<dyn UIContentPropertyEvent>,
    ) {
        let Some(event) = event.downcast_ref::<EEventType>() else {
            return;
        };
        let mut content = content.borrow_mut();
        let Some(static_mesh) = content.downcast_mut::<StaticMesh>() else {
            return;
        };
        let Some(project_context) = editor_context.project_context_mut() else {
            return;
        };

        match event {
            EEventType::UpdateStaticMeshEnableMultiresolution(new_value) => {
                static_mesh.is_enable_multiresolution = *new_value;
                if let Err(err) =
                    Self::create_multi_res_mesh_cache_non_blocking(project_context, &static_mesh)
                {
                    log::warn!("{}", err);
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
        let _ = engine;
        let content = TypedContent::<rs_engine::content::static_mesh::StaticMesh>::new(content)
            .expect("Matched type");
        if let Some(future) = crate::load_content::load_static_mesh::LoadStaticMesh::new(
            loading_context.clone(),
            content,
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
        let static_mesh = TypedContent::<StaticMesh>::new(content).expect("Matched type");
        editor_context
            .open_static_mesh_window(&mut static_mesh.borrow_mut(), event_loop_window_target);
    }

    fn export(
        &self,
        content: SingleThreadMutType<Box<dyn rs_content::Content>>,
        artifact_asset_encoder: &mut rs_artifact::artifact::ArtifactAssetEncoder,
        associated_assets: &mut HashMap<url::Url, Box<dyn Asset>>,
        model_loader: &mut ModelLoader,
        project_context: &ProjectContext,
    ) -> anyhow::Result<()> {
        let asset_folder_path = project_context.get_asset_folder_path();
        let static_mesh = TypedContent::<StaticMesh>::new(content).expect("Matched type");
        let static_mesh = static_mesh.borrow();
        {
            let file_path = asset_folder_path.join(&static_mesh.asset_info.relative_path);
            model_loader
                .load_scene_from_file_and_cache(&file_path)
                .unwrap();
            let loaded_static_mesh = model_loader
                .to_runtime_cache_static_mesh(
                    &static_mesh,
                    &asset_folder_path,
                    ResourceManager::default(),
                )
                .expect("Loaded");
            associated_assets.insert(
                loaded_static_mesh.url.clone(),
                Box::new(loaded_static_mesh.deref().clone()),
            );
        }
        artifact_asset_encoder.encode_content(&*static_mesh);
        Ok(())
    }
}
