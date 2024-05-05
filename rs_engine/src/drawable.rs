use crate::handle::BufferHandle;
use rs_render::command::EBindingResource;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub enum EDrawObjectType {
    Static(StaticMeshDrawObject),
    Skin(SkinMeshDrawObject),
    SkinMaterial(MaterialDrawObject),
}

#[derive(Clone, Debug)]
pub struct StaticMeshDrawObject {
    pub(crate) id: u32,
    pub(crate) vertex_buffers: Vec<BufferHandle>,
    pub(crate) vertex_count: u32,
    pub(crate) index_buffer: Option<BufferHandle>,
    pub(crate) index_count: Option<u32>,
    pub(crate) global_binding_resources: Vec<EBindingResource>,
    pub(crate) vt_binding_resources: Vec<EBindingResource>,
    pub(crate) binding_resources: Vec<Vec<EBindingResource>>,
    pub(crate) render_pipeline: String,
    pub(crate) constants_buffer_handle: BufferHandle,
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
    pub(crate) global_binding_resources: Vec<EBindingResource>,
    pub(crate) vt_binding_resources: Vec<EBindingResource>,
    pub(crate) binding_resources: Vec<Vec<EBindingResource>>,
    pub(crate) render_pipeline: String,
    pub(crate) constants_buffer_handle: BufferHandle,
    pub constants: rs_render::render_pipeline::skin_mesh_shading::Constants,
    pub diffuse_texture_url: Option<url::Url>,
    pub specular_texture_url: Option<url::Url>,
}

#[derive(Clone, Debug)]
pub struct MaterialDrawObject {
    pub(crate) id: u32,
    pub(crate) vertex_buffers: Vec<BufferHandle>,
    pub(crate) vertex_count: u32,
    pub(crate) index_buffer: Option<BufferHandle>,
    pub(crate) index_count: Option<u32>,
    pub(crate) global_binding_resources: Vec<EBindingResource>,
    pub(crate) vt_binding_resources: Vec<EBindingResource>,
    pub(crate) binding_resources: Vec<Vec<EBindingResource>>,
    pub(crate) material: Rc<RefCell<crate::content::material::Material>>,
    pub(crate) constants_buffer_handle: BufferHandle,
    pub constants: rs_render::render_pipeline::material_pipeline::Constants,
}
