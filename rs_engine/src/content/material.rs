use crate::engine::Engine;
use crate::handle::BufferHandle;
use crate::uniform_map::UniformMap;
use crate::{handle::MaterialRenderPipelineHandle, url_extension::UrlExtension};
use rs_artifact::material::MaterialInfo;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use rs_render_types::MaterialOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ParamentResource {
    pub handle: BufferHandle,
    pub uniform_map: UniformMap,
}

#[derive(Clone, Debug)]
struct MaterialRuntime {
    pipeline_handle: MaterialRenderPipelineHandle,
    material_info: HashMap<MaterialOptions, MaterialInfo>,
    parament_resources: HashMap<MaterialOptions, Vec<ParamentResource>>,
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
                material_info: HashMap::new(),
                parament_resources: HashMap::new(),
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

    pub fn set_material_info(
        &mut self,
        engine: &mut Engine,
        material_info: HashMap<MaterialOptions, MaterialInfo>,
    ) {
        if let Some(runtime) = self.run_time.as_mut() {
            runtime.material_info = material_info;
            self.on_material_info_changed(engine);
        }
    }

    pub fn get_material_info(&self) -> &HashMap<MaterialOptions, MaterialInfo> {
        if let Some(runtime) = self.run_time.as_ref() {
            &runtime.material_info
        } else {
            panic!()
        }
    }

    pub fn recreate_paraments(&mut self, engine: &mut Engine) {
        let Some(runtime) = self.run_time.as_mut() else {
            return;
        };
        let name = self.url.get_name_in_editor();
        for (material_options, material_info) in &runtime.material_info {
            for paramenter in &material_info.paramenters {
                if !paramenter.is_valid() {
                    continue;
                }
                let uniform_map = UniformMap::new(&paramenter.fields);
                let buffer_handle = engine
                    .create_buffer(
                        uniform_map.get_data().to_vec(),
                        wgpu::BufferUsages::UNIFORM,
                        Some(name.clone()),
                    )
                    .ok();
                if let Some(buffer_handle) = buffer_handle {
                    runtime
                        .parament_resources
                        .entry(material_options.clone())
                        .or_default()
                        .push(ParamentResource {
                            handle: buffer_handle,
                            uniform_map,
                        });
                }
            }
        }
    }

    pub fn on_material_info_changed(&mut self, engine: &mut Engine) {
        self.reset_paraments();
        self.recreate_paraments(engine);
    }

    pub fn reset_paraments(&mut self) {
        if let Some(runtime) = self.run_time.as_mut() {
            runtime.parament_resources.clear();
        }
    }

    pub fn default_parament_handle(&self, options: &MaterialOptions) -> Option<BufferHandle> {
        let runtime = self.run_time.as_ref()?;
        let resource = runtime.parament_resources.get(options)?.get(0)?.clone();
        Some(resource.handle)
    }
}
