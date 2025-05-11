use super::types::{PostLoading, PostLoadingContext, PreLoadingContext};
use crate::impl_default_load_future;
use crate::impl_default_load_future_body;
use rs_artifact::asset::Asset;
use rs_engine::thread_pool::ThreadPool;
use rs_foundation::new::SingleThreadMutType;
use std::sync::Arc;

type ResultType = Result<rs_artifact::sound::Sound, anyhow::Error>;

pub struct LoadSound<'a> {
    _loading_context: PreLoadingContext<'a>,
    _content: SingleThreadMutType<rs_engine::content::sound::Sound>,
    receiver: std::sync::mpsc::Receiver<ResultType>,
    resource: Option<ResultType>,
}

impl_default_load_future!(LoadSound<'a>);

impl<'a> PostLoading for LoadSound<'a> {
    fn on_loading_finished(&mut self, context: PostLoadingContext) {
        let sound_resource = match self.resource.take().expect("Not null") {
            Ok(sound_resource) => sound_resource,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };

        let rm = context.resource_manager;
        let url = sound_resource.get_url();
        rm.add_sound(url, Arc::new(sound_resource));
    }
}

impl<'a> LoadSound<'a> {
    pub fn new(
        loading_context: PreLoadingContext<'a>,
        content: SingleThreadMutType<rs_engine::content::sound::Sound>,
    ) -> Option<LoadSound<'a>> {
        let maintain = content.clone();
        let sound = content.borrow_mut();
        let url = sound.asset_info.get_url();
        let path = loading_context
            .project_context
            .get_asset_folder_path()
            .join(&sound.asset_info.relative_path);
        let (sender, receiver) = std::sync::mpsc::channel();
        ThreadPool::global().spawn({
            move || {
                let data = std::fs::read(path);
                match data {
                    Ok(data) => {
                        let sound_resouce = rs_artifact::sound::Sound {
                            url: url.clone(),
                            sound_file_type: rs_artifact::sound::ESoundFileType::Unknow,
                            data,
                        };
                        let _ = sender.send(Ok(sound_resouce));
                    }
                    Err(err) => {
                        let _ = sender.send(Err(err.into()));
                    }
                }
            }
        });
        Some(Self {
            _loading_context: loading_context,
            _content: maintain,
            receiver,
            resource: None,
        })
    }
}
