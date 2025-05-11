use super::types::SceneWrapper;
use super::types::{PostLoading, PostLoadingContext, PreLoadingContext};
use crate::impl_default_load_future;
use crate::impl_default_load_future_body;
use anyhow::anyhow;
use anyhow::Context;
use rs_engine::thread_pool::ThreadPool;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_model_loader::model_loader::ModelLoader;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

type ResultType = Result<rs_artifact::skin_mesh::SkinMesh, anyhow::Error>;

pub(crate) struct LoadSkeletonMesh<'a> {
    _loading_context: PreLoadingContext<'a>,
    _content: SingleThreadMutType<rs_engine::content::skeleton_mesh::SkeletonMesh>,
    receiver: std::sync::mpsc::Receiver<ResultType>,
    resource: Option<ResultType>,
}

impl_default_load_future!(LoadSkeletonMesh<'a>);

impl<'a> PostLoading for LoadSkeletonMesh<'a> {
    fn on_loading_finished(&mut self, context: PostLoadingContext) {
        let skeleton_mesh_resource = match self.resource.take().expect("Not null") {
            Ok(skin_mesh_resource) => skin_mesh_resource,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };
        context.resource_manager.add_skin_mesh(
            skeleton_mesh_resource.url.clone(),
            Arc::new(skeleton_mesh_resource),
        );
    }
}

impl<'a> LoadSkeletonMesh<'a> {
    pub fn new(
        loading_context: PreLoadingContext<'a>,
        content: SingleThreadMutType<rs_engine::content::skeleton_mesh::SkeletonMesh>,
        scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>>,
    ) -> Option<LoadSkeletonMesh<'a>> {
        let maintain = content.clone();
        let skeleton_mesh = content.borrow();
        let file_path = loading_context
            .project_context
            .get_project_folder_path()
            .join(&skeleton_mesh.get_relative_path());
        let name = skeleton_mesh.get_skeleton_mesh_name().clone();
        let url = skeleton_mesh.asset_url.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        ThreadPool::global().spawn(move || {
            let result = (|| {
                let scene = super::types::load_scene(file_path, scenes)?;
                let mesh = scene
                    .meshes
                    .iter()
                    .find(|x| x.borrow().name == name)
                    .context(anyhow!("Can not find mesh"))?;
                let mesh = ModelLoader::to_artifact_skin_mesh(&mesh.borrow(), name, url);
                anyhow::Ok(mesh)
            })();
            let _ = sender.send(result);
        });
        Some(Self {
            _loading_context: loading_context,
            _content: maintain,
            receiver,
            resource: None,
        })
    }
}
