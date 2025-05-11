use super::types::SceneWrapper;
use super::types::{PostLoading, PostLoadingContext, PreLoadingContext};
use crate::impl_default_load_future;
use crate::impl_default_load_future_body;
use anyhow::Context;
use rs_engine::thread_pool::ThreadPool;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_model_loader::model_loader::ModelLoader;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

type ResultType = Result<rs_artifact::skeleton_animation::SkeletonAnimation, anyhow::Error>;

pub(crate) struct LoadSkeletonAnimation<'a> {
    _loading_context: PreLoadingContext<'a>,
    _content: SingleThreadMutType<rs_engine::content::skeleton_animation::SkeletonAnimation>,
    receiver: std::sync::mpsc::Receiver<ResultType>,
    resource: Option<ResultType>,
}

impl_default_load_future!(LoadSkeletonAnimation<'a>);

impl<'a> PostLoading for LoadSkeletonAnimation<'a> {
    fn on_loading_finished(&mut self, context: PostLoadingContext) {
        let skeleton_animation_resource = match self.resource.take().expect("Not null") {
            Ok(skeleton_animation_resource) => skeleton_animation_resource,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };
        context.resource_manager.add_skeleton_animation(
            skeleton_animation_resource.url.clone(),
            Arc::new(skeleton_animation_resource),
        );
    }
}

impl<'a> LoadSkeletonAnimation<'a> {
    pub fn new(
        loading_context: PreLoadingContext<'a>,
        content: SingleThreadMutType<rs_engine::content::skeleton_animation::SkeletonAnimation>,
        scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>>,
    ) -> Option<LoadSkeletonAnimation<'a>> {
        let maintain = content.clone();
        let skeleton_animation = content.borrow();
        let file_path = loading_context
            .project_context
            .get_project_folder_path()
            .join(&skeleton_animation.get_relative_path());
        let name = skeleton_animation.get_animation_name().clone();
        let url = skeleton_animation.asset_url.clone();
        let scenes = scenes.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        ThreadPool::global().spawn(move || {
            let result = (|| {
                let scene = super::types::load_scene(file_path, scenes)?;
                let animation = scene
                    .animations
                    .iter()
                    .find(|x| x.name == name)
                    .context(anyhow::anyhow!("Can not find animation"))?;
                let skeleton_animation =
                    ModelLoader::to_artifact_skeleton_animation(&animation, name, url);
                anyhow::Ok(skeleton_animation)
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
