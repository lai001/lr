use crate::{handle::MaterialRenderPipelineHandle, url_extension::UrlExtension};
use rs_artifact::material::MaterialInfo;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
struct MaterialRuntime {
    pipeline_handle: MaterialRenderPipelineHandle,
    material_info: MaterialInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Material {
    pub url: url::Url,
    pub asset_url: url::Url,
    #[serde(skip)]
    run_time: Option<MaterialRuntime>,
}

impl Asset for Material {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::Material)
    }
}

impl Material {
    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    pub fn new(url: url::Url, asset_url: url::Url) -> Material {
        Material {
            url,
            asset_url,
            run_time: None,
        }
    }

    pub fn set_pipeline_handle(&mut self, pipeline_handle: MaterialRenderPipelineHandle) {
        if let Some(runtime) = self.run_time.as_mut() {
            runtime.pipeline_handle = pipeline_handle;
        } else {
            self.run_time = Some(MaterialRuntime {
                pipeline_handle,
                material_info: MaterialInfo::default(),
            });
        }
    }

    pub fn get_pipeline_handle(&self) -> Option<MaterialRenderPipelineHandle> {
        if let Some(runtime) = self.run_time.as_ref() {
            Some(runtime.pipeline_handle.clone())
        } else {
            None
        }
    }

    pub fn set_material_info(&mut self, material_info: MaterialInfo) {
        if let Some(runtime) = self.run_time.as_mut() {
            runtime.material_info = material_info;
        }
    }

    pub fn get_material_info(&self) -> &MaterialInfo {
        if let Some(runtime) = self.run_time.as_ref() {
            &runtime.material_info
        } else {
            panic!()
        }
    }
}
