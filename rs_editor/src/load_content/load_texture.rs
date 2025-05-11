use super::types::{PostLoading, PostLoadingContext, PreLoadingContext};
use crate::impl_default_load_future;
use crate::impl_default_load_future_body;
use anyhow::Context;
use rs_engine::thread_pool::ThreadPool;
use rs_foundation::new::SingleThreadMutType;

type ResultType = Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, anyhow::Error>;

pub struct LoadTexture<'a> {
    _loading_context: PreLoadingContext<'a>,
    _content: SingleThreadMutType<rs_engine::content::texture::TextureFile>,
    receiver: std::sync::mpsc::Receiver<ResultType>,
    resource: Option<ResultType>,
}

impl_default_load_future!(LoadTexture<'a>);

impl<'a> PostLoading for LoadTexture<'a> {
    fn on_loading_finished(&mut self, context: PostLoadingContext) {
        let image_resouce = match self.resource.take().expect("Not null") {
            Ok(image_resouce) => image_resouce,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };
        if let Err(err) = context
            .engine
            .create_texture_from_image(&self._content.borrow().url, &image_resouce)
        {
            log::warn!("{}", err);
        }
    }
}

impl<'a> LoadTexture<'a> {
    pub fn new(
        loading_context: PreLoadingContext<'a>,
        content: SingleThreadMutType<rs_engine::content::texture::TextureFile>,
    ) -> Option<LoadTexture<'a>> {
        let maintain = content.clone();
        let texture_file = content.borrow_mut();
        let Some(image_reference) = &texture_file.get_image_reference_path() else {
            return None;
        };
        let absolute_path = loading_context
            .project_context
            .get_project_folder_path()
            .join(image_reference);
        let (sender, receiver) = std::sync::mpsc::channel();
        ThreadPool::global().spawn({
            move || {
                let image = (|| {
                    let dynamic_image = image::open(&absolute_path)
                        .context(anyhow::anyhow!("Can not open file, {:?}", &absolute_path))?;
                    let image = match dynamic_image {
                        image::DynamicImage::ImageRgba8(image) => image,
                        x => x.to_rgba8(),
                    };
                    return Ok(image);
                })();
                let _ = sender.send(image);
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
