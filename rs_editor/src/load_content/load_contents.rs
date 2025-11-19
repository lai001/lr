use super::types::{PreLoadingContext, SceneWrapper};
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
        files: &Vec<EContentFileType>,
    ) -> anyhow::Result<()> {
        let _span = tracy_client::span!();
        {
            let resource_manager = engine.get_resource_manager().clone();
            let futures: FuturesUnordered<Box<dyn super::types::PostLoading<Output = ()>>> =
                FuturesUnordered::new();
            let cx = PreLoadingContext {
                resource_manager: &resource_manager,
                project_context,
            };
            let scenes: MultipleThreadMutType<HashMap<PathBuf, SceneWrapper>> =
                MultipleThreadMut::new(HashMap::new());

            for file in files {
                match file {
                    EContentFileType::StaticMesh(ref_cell) => {
                        if let Some(future) = super::load_static_mesh::LoadStaticMesh::new(
                            cx.clone(),
                            ref_cell.clone(),
                            scenes.clone(),
                        ) {
                            futures.push(Box::new(future));
                        }
                    }
                    EContentFileType::SkeletonMesh(ref_cell) => {
                        if let Some(future) = super::load_skeleton_mesh::LoadSkeletonMesh::new(
                            cx.clone(),
                            ref_cell.clone(),
                            scenes.clone(),
                        ) {
                            futures.push(Box::new(future));
                        }
                    }
                    EContentFileType::SkeletonAnimation(ref_cell) => {
                        if let Some(future) =
                            super::load_skeleton_animation::LoadSkeletonAnimation::new(
                                cx.clone(),
                                ref_cell.clone(),
                                scenes.clone(),
                            )
                        {
                            futures.push(Box::new(future));
                        }
                    }
                    EContentFileType::Skeleton(ref_cell) => {
                        if let Some(future) = super::load_skeleton::LoadSkeleton::new(
                            cx.clone(),
                            ref_cell.clone(),
                            scenes.clone(),
                        ) {
                            futures.push(Box::new(future));
                        }
                    }
                    EContentFileType::Texture(ref_cell) => {
                        if let Some(future) =
                            super::load_texture::LoadTexture::new(cx.clone(), ref_cell.clone())
                        {
                            futures.push(Box::new(future));
                        }
                        if let Some(future) = super::load_virtual_texture::LoadVirtualTexture::new(
                            cx.clone(),
                            ref_cell.clone(),
                        ) {
                            futures.push(Box::new(future));
                        }
                    }
                    EContentFileType::Level(_) => {}
                    EContentFileType::Material(ref_cell) => {
                        if let Some(future) =
                            super::load_material::LoadMaterial::new(cx.clone(), ref_cell.clone())
                        {
                            futures.push(Box::new(future));
                        }
                    }
                    EContentFileType::IBL(ref_cell) => {
                        if let Some(future) =
                            super::load_ibl::LoadIBL::new(cx.clone(), ref_cell.clone())
                        {
                            futures.push(Box::new(future));
                        }
                    }
                    EContentFileType::ParticleSystem(_) => {}
                    EContentFileType::Sound(ref_cell) => {
                        if let Some(future) =
                            super::load_sound::LoadSound::new(cx.clone(), ref_cell.clone())
                        {
                            futures.push(Box::new(future));
                        }
                    }
                    EContentFileType::Curve(_) => {}
                    EContentFileType::BlendAnimations(_) => {}
                    EContentFileType::MaterialParamentersCollection(ref_cell) => {
                        let mut material_paramenters_collection = ref_cell.borrow_mut();
                        material_paramenters_collection.initialize(engine);
                    }
                    EContentFileType::RenderTarget2D(render_target_2d) => {
                        let mut render_target_2d = render_target_2d.borrow_mut();
                        render_target_2d.init_resouce(engine);
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
