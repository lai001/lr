use super::types::{PreLoadingContext, SceneWrapper};
use crate::content_edit::ContentEdit;
use crate::project_context::ProjectContext;
use futures::stream::FuturesUnordered;
use rs_engine::content::content_file_type::EContentFileType;
use rs_foundation::new::{MultipleThreadMut, MultipleThreadMutType};
use rs_model_loader::model_loader::ModelLoader;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

pub struct LoadContents {}

impl LoadContents {
    pub fn load(
        engine: &mut rs_engine::engine::Engine,
        project_context: &ProjectContext,
        model_loader: &mut ModelLoader,
        files: Vec<EContentFileType>,
        content_edit: &mut ContentEdit,
    ) -> anyhow::Result<()> {
        let module_manager = project_context.module_manager.clone();
        let content_manager = project_context.content_manager.clone();
        let _span = tracy_client::span!();
        {
            let resource_manager = engine.get_resource_manager().clone();
            let futures: FuturesUnordered<Box<dyn super::types::PostLoading<Output = ()>>> =
                FuturesUnordered::new();
            let cx = PreLoadingContext {
                resource_manager: &resource_manager,
                project_context,
                module_manager,
                content_manager,
            };
            let scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>> =
                MultipleThreadMut::new(HashMap::new());

            for file in files {
                let content = file.clone();
                let editable = {
                    let content_ref = content.borrow();
                    content_edit.editable(content_ref.as_ref())
                };
                if let Some(editable) = editable {
                    let scenes_for_async = scenes.clone();
                    let async_futures =
                        editable.load_async(content.clone(), cx.clone(), scenes_for_async, engine);
                    if async_futures.is_empty() {
                        editable.load_sync(content.clone(), cx.clone(), scenes.clone(), engine);
                    } else {
                        for future in async_futures {
                            futures.push(future);
                        }
                    }
                }
            }
            let rt = tokio::runtime::Builder::new_current_thread().build()?;
            rt.block_on(async {
                for mut fut in futures {
                    {
                        let refence = &mut fut;
                        tokio::pin!(refence);
                        refence.await;
                    }
                    let context = crate::load_content::types::PostLoadingContext {
                        engine,
                        project_context,
                        resource_manager: &resource_manager,
                    };
                    fut.on_loading_finished(context);
                }
            });

            // while Arc::strong_count(&scenes) != 1 {}
            let scenes = Arc::try_unwrap(scenes).expect("Exactly one strong reference");
            let inner = scenes.into_inner().expect("Return the underlying data");
            for (file_path, scene) in inner {
                // while Arc::strong_count(&scene.0) != 1 {}
                let scene = Arc::try_unwrap(scene.0).expect("Exactly one strong reference");
                model_loader.cache_scene(&file_path, scene);
            }
            Ok(())
        }
    }
}
