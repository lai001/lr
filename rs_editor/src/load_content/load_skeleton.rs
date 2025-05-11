use super::types::SceneWrapper;
use super::types::{PostLoading, PostLoadingContext, PreLoadingContext};
use crate::impl_default_load_future;
use crate::impl_default_load_future_body;
use rs_engine::thread_pool::ThreadPool;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_model_loader::model_loader::ModelLoader;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

type ResultType = Result<rs_artifact::skeleton::Skeleton, anyhow::Error>;

pub(crate) struct LoadSkeleton<'a> {
    _loading_context: PreLoadingContext<'a>,
    _content: SingleThreadMutType<rs_engine::content::skeleton::Skeleton>,
    receiver: std::sync::mpsc::Receiver<ResultType>,
    resource: Option<ResultType>,
}

impl_default_load_future!(LoadSkeleton<'a>);

impl<'a> PostLoading for LoadSkeleton<'a> {
    fn on_loading_finished(&mut self, context: PostLoadingContext) {
        let skeleton_resource = match self.resource.take().expect("Not null") {
            Ok(skeleton_resource) => skeleton_resource,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };
        context
            .resource_manager
            .add_skeleton(skeleton_resource.url.clone(), Arc::new(skeleton_resource));
    }
}

impl<'a> LoadSkeleton<'a> {
    pub fn new(
        loading_context: PreLoadingContext<'a>,
        content: SingleThreadMutType<rs_engine::content::skeleton::Skeleton>,
        scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>>,
    ) -> Option<LoadSkeleton<'a>> {
        let maintain = content.clone();
        let skeleton = content.borrow();
        let file_path = loading_context
            .project_context
            .get_project_folder_path()
            .join(&skeleton.get_relative_path());
        let url = skeleton.asset_url.clone();
        let scenes = scenes.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        ThreadPool::global().spawn(move || {
            let result = (|| {
                let scene = super::types::load_scene(file_path, scenes)?;
                let armature = scene.armatures.values().next().unwrap().clone();
                let root_node = scene.root_node.clone().unwrap();
                let name = armature.borrow().name.clone();
                let skeleton = ModelLoader::to_artifact_skeleton(armature, root_node, name, url);
                anyhow::Ok(skeleton)
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
