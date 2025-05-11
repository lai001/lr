use super::types::SceneWrapper;
use super::types::{PostLoading, PostLoadingContext, PreLoadingContext};
use crate::impl_default_load_future;
use crate::impl_default_load_future_body;
use anyhow::anyhow;
use anyhow::Context;
use rs_engine::thread_pool::ThreadPool;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_metis::cluster::ClusterCollection;
use rs_model_loader::model_loader::ModelLoader;
use rs_render::command::{CreateMultipleResolutionMesh, RenderCommand};
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

struct Output {
    cluster_collection: Option<Result<ClusterCollection, anyhow::Error>>,
    static_mesh: Result<rs_artifact::static_mesh::StaticMesh, anyhow::Error>,
}

type ResultType = Output;

pub(crate) struct LoadStaticMesh<'a> {
    _loading_context: PreLoadingContext<'a>,
    content: SingleThreadMutType<rs_engine::content::static_mesh::StaticMesh>,
    receiver: std::sync::mpsc::Receiver<ResultType>,
    resource: Option<ResultType>,
}

impl_default_load_future!(LoadStaticMesh<'a>);

impl<'a> PostLoading for LoadStaticMesh<'a> {
    fn on_loading_finished(&mut self, context: PostLoadingContext) {
        let output = self.resource.take().expect("Not null");

        let static_mesh = match output.static_mesh {
            Ok(static_mesh) => static_mesh,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };
        let static_mesh = Arc::new(static_mesh);
        if let Some(cluster_collection) = output.cluster_collection {
            match cluster_collection {
                Ok(cluster_collection) => {
                    let static_mesh_content = self.content.borrow_mut();
                    let debug_label = static_mesh_content.get_name();
                    let url = static_mesh.url.clone();
                    let rm = context.resource_manager;
                    let handle = rm.next_multiple_resolution_mesh_handle(url);
                    context
                        .engine
                        .send_render_command(RenderCommand::CreateMultiResMesh(
                            CreateMultipleResolutionMesh {
                                handle: *handle,
                                vertexes: static_mesh.vertexes.clone(),
                                indices: static_mesh.indexes.clone(),
                                debug_label: Some(debug_label),
                                cluster_collection,
                            },
                        ));
                }
                Err(err) => {
                    log::warn!("{}", err);
                }
            }
        }
        context
            .resource_manager
            .add_static_mesh(static_mesh.url.clone(), static_mesh.clone());
    }
}

impl<'a> LoadStaticMesh<'a> {
    pub fn new(
        loading_context: PreLoadingContext<'a>,
        content: SingleThreadMutType<rs_engine::content::static_mesh::StaticMesh>,
        scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>>,
    ) -> Option<LoadStaticMesh<'a>> {
        let maintain = content.clone();
        let static_mesh = content.borrow();
        let file_path = loading_context
            .project_context
            .get_asset_folder_path()
            .join(&static_mesh.asset_info.relative_path);
        let name = static_mesh.asset_info.path.clone();
        let url = static_mesh.asset_info.get_url();
        let cache_filename = static_mesh.get_name();
        let mesh_cluster_dir = loading_context.project_context.get_mesh_cluster_dir();
        let cache_path = mesh_cluster_dir.join(cache_filename);
        let is_enable_multiresolution = static_mesh.is_enable_multiresolution;
        let (sender, receiver) = std::sync::mpsc::channel();
        ThreadPool::global().spawn(move || {
            let result = (|| {
                let scene = super::types::load_scene(file_path, scenes)?;
                let mesh = scene
                    .meshes
                    .iter()
                    .find(|x| x.borrow().name == name)
                    .context(anyhow!("Can not find mesh"))?;
                let static_mesh = ModelLoader::to_artifact_static_mesh(&mesh.borrow(), name, url);
                anyhow::Ok(static_mesh)
            })();
            let cluster_collection = if is_enable_multiresolution {
                let result = (|| {
                    let read_bytes = std::fs::read(&cache_path)?;
                    let cluster_collection = rs_artifact::bincode_legacy::deserialize::<
                        ClusterCollection,
                    >(&read_bytes, None)?;
                    anyhow::Ok(cluster_collection)
                })();
                Some(result)
            } else {
                None
            };

            let _ = sender.send(Output {
                cluster_collection,
                static_mesh: result,
            });
        });
        Some(Self {
            _loading_context: loading_context,
            content: maintain,
            receiver,
            resource: None,
        })
    }
}
