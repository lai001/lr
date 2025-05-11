use super::types::{PostLoading, PostLoadingContext, PreLoadingContext};
use crate::{
    impl_default_load_future, impl_default_load_future_body, project_context::ProjectContext,
};
use rs_engine::thread_pool::ThreadPool;
use rs_foundation::new::SingleThreadMutType;

type ResultType = Result<rs_artifact::ibl_baking::IBLBaking, anyhow::Error>;

pub struct LoadIBL<'a> {
    _loading_context: PreLoadingContext<'a>,
    _content: SingleThreadMutType<rs_engine::content::ibl::IBL>,
    receiver: std::sync::mpsc::Receiver<ResultType>,
    resource: Option<ResultType>,
}

impl_default_load_future!(LoadIBL<'a>);

impl<'a> PostLoading for LoadIBL<'a> {
    fn on_loading_finished(&mut self, context: PostLoadingContext) {
        let ibl_resource = match self.resource.take().expect("Not null") {
            Ok(ibl_resource) => ibl_resource,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };

        context
            .engine
            .upload_prebake_ibl(ibl_resource.url.clone(), ibl_resource);
    }
}

impl<'a> LoadIBL<'a> {
    pub fn new(
        loading_context: PreLoadingContext<'a>,
        content: SingleThreadMutType<rs_engine::content::ibl::IBL>,
    ) -> Option<LoadIBL<'a>> {
        let maintain = content.clone();
        let ibl = content.borrow();
        let url = ibl.url.clone();
        let name = rs_engine::url_extension::UrlExtension::get_name_in_editor(&url);
        let image_reference = ibl.image_reference.clone();
        let project_folder_path = loading_context
            .project_context
            .get_project_folder_path()
            .clone();
        let Some(image_reference) = image_reference else {
            return None;
        };
        let (sender, receiver) = std::sync::mpsc::channel();
        ThreadPool::global().spawn({
            move || {
                let result: anyhow::Result<rs_artifact::ibl_baking::IBLBaking> = (|| {
                    let sub_folder = image_reference;
                    let ibl_bake_cache_dir =
                        ProjectContext::make_ibl_bake_cache_dir(&project_folder_path, &sub_folder);
                    let brdf_data_path = ibl_bake_cache_dir.join("brdf.dds");
                    let pre_filter_data_path = ibl_bake_cache_dir.join("pre_filter.dds");
                    let irradiance_data_path = ibl_bake_cache_dir.join("irradiance.dds");
                    let brdf_data = std::fs::read(brdf_data_path)?;
                    let pre_filter_data = std::fs::read(pre_filter_data_path)?;
                    let irradiance_data = std::fs::read(irradiance_data_path)?;
                    let ibl_baking = rs_artifact::ibl_baking::IBLBaking {
                        name,
                        url,
                        brdf_data,
                        pre_filter_data,
                        irradiance_data,
                    };
                    Ok(ibl_baking)
                })(
                );
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
