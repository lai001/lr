use anyhow::Context;
use rs_engine::resource_manager::ResourceManager;
use rs_foundation::new::MultipleThreadMutType;
use rs_model_loader::model_loader::ModelLoader;
use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::Arc};

pub struct PostLoadingContext<'a> {
    pub engine: &'a mut rs_engine::engine::Engine,
    pub project_context: &'a crate::project_context::ProjectContext,
    pub resource_manager: &'a ResourceManager,
}

pub trait PostLoading: std::future::Future + Unpin {
    fn on_loading_finished(&mut self, context: PostLoadingContext);
}

#[derive(Clone)]
pub struct PreLoadingContext<'a> {
    pub resource_manager: &'a ResourceManager,
    pub project_context: &'a crate::project_context::ProjectContext,
}

pub(crate) struct SceneWrapper(pub(crate) Arc<rs_assimp::scene::Scene<'static>>);

impl Debug for SceneWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SceneWrapper").field(&self.0.name).finish()
    }
}
unsafe impl Send for SceneWrapper {}

pub(crate) fn load_scene(
    file_path: PathBuf,
    scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>>,
) -> anyhow::Result<Arc<rs_assimp::scene::Scene<'static>>> {
    let mut scenes = scenes.lock().unwrap();
    if !scenes.contains_key(&file_path) {
        let scene =
            ModelLoader::load_scene_from_file(&file_path).map(|x| SceneWrapper(Arc::new(x)))?;
        scenes.insert(file_path.clone(), scene);
    }
    scenes
        .get(&file_path)
        .map(|x| x.0.clone())
        .context(anyhow::anyhow!(
            "Failed to load scene from file, {:?}",
            &file_path
        ))
}

#[macro_export]
macro_rules! impl_default_load_future_body {
    ($receiver:ident, $resource:ident) => {
        type Output = ();

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            match self.$receiver.try_recv() {
                Ok(result) => {
                    self.$resource = Some(result);
                }
                Err(err) => {
                    if let std::sync::mpsc::TryRecvError::Disconnected = err {
                        log::warn!("{}", std::sync::mpsc::TryRecvError::Disconnected);
                    }
                }
            }
            cx.waker().wake_by_ref();
            if self.$resource.is_some() {
                std::task::Poll::Ready(())
            } else {
                std::task::Poll::Pending
            }
        }
    };
}

#[macro_export]
macro_rules! impl_default_load_future {
    ($struct:ident, $receiver:ident, $resource:ident) => {
        impl std::future::Future for $struct {
            impl_default_load_future_body!($receiver, $resource);
        }
    };
    ($struct:ident<$lt:lifetime>, $receiver:ident, $resource:ident) => {
        impl<$lt> std::future::Future for $struct<$lt> {
            impl_default_load_future_body!($receiver, $resource);
        }
    };
    ($struct:ident<$lt:lifetime>) => {
        impl<$lt> std::future::Future for $struct<$lt> {
            impl_default_load_future_body!(receiver, resource);
        }
    };
    ($struct:ident) => {
        impl std::future::Future for $struct {
            impl_default_load_future_body!(receiver, resource);
        }
    };
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    struct LoadA<'a> {
        receiver: std::sync::mpsc::Receiver<()>,
        resource: Option<()>,
        _marker: PhantomData<&'a ()>,
    }
    impl_default_load_future!(LoadA<'a>);

    struct LoadB {
        receiver: std::sync::mpsc::Receiver<()>,
        resource: Option<()>,
    }
    impl_default_load_future!(LoadB);

    struct LoadC<'a> {
        receiver: std::sync::mpsc::Receiver<()>,
        resource: Option<()>,
        _marker: PhantomData<&'a ()>,
    }
    impl_default_load_future!(LoadC<'a>, receiver, resource);

    struct LoadD {
        receiver: std::sync::mpsc::Receiver<()>,
        resource: Option<()>,
    }
    impl_default_load_future!(LoadD, receiver, resource);
}
