use crate::{handle::MaterialRenderPipelineHandle, url_extension::UrlExtension};
use rs_artifact::material::TextureBinding;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug)]
struct MaterialRuntime {
    pipeline_handle: MaterialRenderPipelineHandle,
    map_textures: HashSet<TextureBinding>,
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
                map_textures: HashSet::new(),
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

    pub fn set_map_textures(&mut self, map_texture_names: HashSet<TextureBinding>) {
        if let Some(runtime) = self.run_time.as_mut() {
            runtime.map_textures = map_texture_names;
        }
    }

    pub fn get_map_textures(&self) -> &HashSet<TextureBinding> {
        if let Some(runtime) = self.run_time.as_ref() {
            &runtime.map_textures
        } else {
            panic!()
        }
    }
}
