use crate::{handle::BufferHandle, player_viewport::PlayerViewport};
use rs_artifact::material::GroupBinding;
use rs_render::command::{DrawObject, EBindingResource};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone)]
pub enum EDrawObjectType {
    Static(StaticMeshDrawObject),
    Skin(SkinMeshDrawObject),
    SkinMaterial(MaterialDrawObject),
    StaticMeshMaterial(StaticMeshMaterialDrawObject),
    Custom(CustomDrawObject),
}

impl EDrawObjectType {
    pub fn switch_player_viewport(&mut self, player_viewport: &PlayerViewport) {
        let resource = EBindingResource::Constants(*player_viewport.global_constants_handle);
        match self {
            EDrawObjectType::Static(draw_object) => {
                draw_object.global_constants_resource = resource;
            }
            EDrawObjectType::Skin(draw_object) => {
                draw_object.global_constants_resource = resource;
            }
            EDrawObjectType::SkinMaterial(draw_object) => {
                draw_object.pbr_binding_resources.global_constants_resource = resource;
            }
            EDrawObjectType::StaticMeshMaterial(draw_object) => {
                draw_object.pbr_binding_resources.global_constants_resource = resource;
            }
            EDrawObjectType::Custom(_) => {
                // unimplemented!()
            }
        }
    }
}

#[derive(Clone)]
pub struct CustomDrawObject {
    pub draw_object: DrawObject,
    pub render_target_type: rs_render::command::ERenderTargetType,
}

#[derive(Clone, Debug)]
pub struct StaticMeshDrawObject {
    pub(crate) id: u32,
    pub(crate) vertex_buffers: Vec<BufferHandle>,
    pub(crate) vertex_count: u32,
    pub(crate) index_buffer: Option<BufferHandle>,
    pub(crate) index_count: Option<u32>,
    pub(crate) global_constants_resource: EBindingResource,
    pub(crate) base_color_sampler_resource: EBindingResource,
    pub(crate) physical_texture_resource: EBindingResource,
    pub(crate) page_table_texture_resource: EBindingResource,
    pub(crate) diffuse_texture_resource: EBindingResource,
    pub(crate) specular_texture_resource: EBindingResource,
    pub(crate) constants_resource: EBindingResource,
    pub(crate) constants_buffer_handle: BufferHandle,
    pub window_id: isize,
    pub constants: rs_render::render_pipeline::shading::Constants,
    pub diffuse_texture_url: Option<url::Url>,
    pub specular_texture_url: Option<url::Url>,
}

#[derive(Clone, Debug)]
pub struct SkinMeshDrawObject {
    pub(crate) id: u32,
    pub(crate) vertex_buffers: Vec<BufferHandle>,
    pub(crate) vertex_count: u32,
    pub(crate) index_buffer: Option<BufferHandle>,
    pub(crate) index_count: Option<u32>,
    pub(crate) global_constants_resource: EBindingResource,
    pub(crate) base_color_sampler_resource: EBindingResource,
    pub(crate) physical_texture_resource: EBindingResource,
    pub(crate) page_table_texture_resource: EBindingResource,
    pub(crate) diffuse_texture_resource: EBindingResource,
    pub(crate) specular_texture_resource: EBindingResource,
    pub(crate) constants_resource: EBindingResource,

    pub(crate) constants_buffer_handle: BufferHandle,
    pub window_id: isize,
    pub constants: rs_render::render_pipeline::skin_mesh_shading::Constants,
    pub diffuse_texture_url: Option<url::Url>,
    pub specular_texture_url: Option<url::Url>,
}

#[derive(Clone, Debug)]
pub struct PBRBindingResources {
    pub(crate) global_constants_resource: EBindingResource,
    pub(crate) base_color_sampler_resource: EBindingResource,
    pub(crate) physical_texture_resource: EBindingResource,
    pub(crate) page_table_texture_resource: EBindingResource,
    pub(crate) brdflut_texture_resource: EBindingResource,
    pub(crate) pre_filter_cube_map_texture_resource: EBindingResource,
    pub(crate) irradiance_texture_resource: EBindingResource,
    pub(crate) shadow_map_texture_resource: EBindingResource,
    pub(crate) constants_resource: EBindingResource,
    pub(crate) virtual_texture_constants_resource: EBindingResource,
    pub(crate) point_lights_constants_resource: EBindingResource,
    pub(crate) spot_lights_constants_resource: EBindingResource,
    pub(crate) cluster_light: Option<EBindingResource>,
    pub(crate) cluster_light_index: Option<EBindingResource>,
    pub(crate) material_parameters_collection_resources: HashMap<GroupBinding, EBindingResource>,
}

#[derive(Clone, Debug)]
pub struct MaterialDrawObject {
    pub(crate) id: u32,
    pub(crate) vertex_buffers: Vec<BufferHandle>,
    pub(crate) vertex_count: u32,
    pub(crate) index_buffer: Option<BufferHandle>,
    pub(crate) index_count: Option<u32>,
    pub(crate) pbr_binding_resources: PBRBindingResources,

    pub(crate) skin_constants_resource: EBindingResource,

    pub(crate) user_textures_resources: Vec<EBindingResource>,

    pub(crate) material: Rc<RefCell<crate::content::material::Material>>,
    pub(crate) constants_buffer_handle: BufferHandle,
    pub(crate) skin_constants_buffer_handle: BufferHandle,
    pub(crate) virtual_texture_constants_buffer_handle: BufferHandle,
    pub window_id: isize,
    pub constants: rs_render::constants::Constants,
    pub skin_constants: rs_render::constants::SkinConstants,
    pub virtual_texture_constants: rs_render::constants::VirtualTextureConstants,

    pub debug_group_label: Option<String>,
}

#[derive(Clone, Debug)]
pub struct StaticMeshMaterialDrawObject {
    pub(crate) id: u32,
    pub(crate) vertex_buffers: Vec<BufferHandle>,
    pub(crate) vertex_count: u32,
    pub(crate) index_buffer: Option<BufferHandle>,
    pub(crate) index_count: Option<u32>,
    pub(crate) pbr_binding_resources: PBRBindingResources,

    pub(crate) user_textures_resources: Vec<EBindingResource>,

    pub(crate) material: Rc<RefCell<crate::content::material::Material>>,
    pub(crate) constants_buffer_handle: BufferHandle,
    pub(crate) virtual_texture_constants_buffer_handle: BufferHandle,

    pub(crate) multiple_resolution_mesh_pass_resource_handle: Option<BufferHandle>,

    pub window_id: isize,
    pub constants: rs_render::constants::Constants,
    pub virtual_texture_constants: rs_render::constants::VirtualTextureConstants,

    pub debug_group_label: Option<String>,
}

// #[derive(Clone, Debug)]
// pub struct MultipleResolutionMeshPassResource {
//     pub mesh_resource_handle: BufferHandle,
//     pub global_constants_resource: EBindingResource,
//     pub constants_resource: EBindingResource,
// }
