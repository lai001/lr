use crate::{asset::Asset, resource_type::EResourceType};
use rs_render_types::MaterialOptions;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Copy)]
pub struct GroupBinding {
    pub group: usize,
    pub binding: usize,
}

impl GroupBinding {
    pub fn new(group: usize, binding: usize) -> Self {
        Self { group, binding }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct TextureBinding {
    pub group: usize,
    pub binding: usize,
    pub texture_url: url::Url,
}

impl TextureBinding {
    pub fn get_texture_bind_name(&self) -> String {
        format!("_texture_{}_{}", self.group, self.binding)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct MaterialParamentersCollectionBinding {
    pub group: usize,
    pub binding: usize,
    pub material_paramenters_collection_url: url::Url,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct MaterialInfo {
    pub map_textures: HashSet<TextureBinding>,
    pub virtual_textures: HashSet<url::Url>,
    pub global_constants_binding: Option<GroupBinding>,
    pub base_color_sampler_binding: Option<GroupBinding>,
    pub physical_texture_binding: Option<GroupBinding>,
    pub page_table_texture_binding: Option<GroupBinding>,
    pub brdflut_texture_binding: Option<GroupBinding>,
    pub pre_filter_cube_map_texture_binding: Option<GroupBinding>,
    pub irradiance_texture_binding: Option<GroupBinding>,
    pub shadow_map_binding: Option<GroupBinding>,
    pub constants_binding: Option<GroupBinding>,
    pub point_lights_binding: Option<GroupBinding>,
    pub spot_lights_binding: Option<GroupBinding>,
    pub skin_constants_binding: Option<GroupBinding>,
    pub virtual_texture_constants_binding: Option<GroupBinding>,
    pub material_paramenters_collection_bindings: HashSet<MaterialParamentersCollectionBinding>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Material {
    pub url: url::Url,
    pub code: HashMap<MaterialOptions, String>,
    pub material_info: HashMap<MaterialOptions, MaterialInfo>,
}

impl Asset for Material {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Material
    }
}
