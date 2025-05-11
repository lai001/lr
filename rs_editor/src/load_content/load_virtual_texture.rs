use super::types::{PostLoading, PostLoadingContext, PreLoadingContext};
use crate::impl_default_load_future;
use crate::impl_default_load_future_body;
use anyhow::Context;
use rs_engine::{
    static_virtual_texture_source::StaticVirtualTextureSource, thread_pool::ThreadPool,
};
use rs_foundation::new::SingleThreadMutType;

type ResultType = Result<StaticVirtualTextureSource, anyhow::Error>;

pub struct LoadVirtualTexture<'a> {
    _loading_context: PreLoadingContext<'a>,
    _content: SingleThreadMutType<rs_engine::content::texture::TextureFile>,
    receiver: std::sync::mpsc::Receiver<ResultType>,
    resource: Option<ResultType>,
}

impl_default_load_future!(LoadVirtualTexture<'a>);

impl<'a> PostLoading for LoadVirtualTexture<'a> {
    fn on_loading_finished(&mut self, context: PostLoadingContext) {
        let virtual_texture = match self.resource.take().expect("Not null") {
            Ok(virtual_texture) => virtual_texture,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };
        context.engine.create_virtual_texture_source(
            self._content.borrow().url.clone(),
            Box::new(virtual_texture),
        );
    }
}

impl<'a> LoadVirtualTexture<'a> {
    pub fn new(
        loading_context: PreLoadingContext<'a>,
        content: SingleThreadMutType<rs_engine::content::texture::TextureFile>,
    ) -> Option<LoadVirtualTexture<'a>> {
        let maintain = content.clone();
        let texture_file = content.borrow_mut();
        let Some(virtual_image_reference) = &texture_file.virtual_image_reference else {
            return None;
        };
        let path = loading_context
            .project_context
            .get_virtual_texture_cache_dir()
            .join(virtual_image_reference);
        let (sender, receiver) = std::sync::mpsc::channel();
        ThreadPool::global().spawn({
            move || {
                let result = StaticVirtualTextureSource::from_file(&path, None)
                    .context(anyhow::anyhow!("Can not load from {:?}", &path));
                let _ = sender.send(result);
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
