use crate::{
    bake_info::BakeInfo, egui_render::EGUIRenderOutput, view_mode::EViewModeType,
    virtual_texture_source::TVirtualTextureSource,
};
use rs_core_minimal::settings::{RenderSettings, VirtualTextureSetting};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};
use wgpu::*;

pub type BufferHandle = u64;
pub type EGUITextureHandle = u64;
pub type TextureHandle = u64;
pub type SamplerHandle = u64;
pub type MaterialRenderPipelineHandle = u64;

#[derive(Clone)]
pub struct TextureDescriptorCreateInfo {
    pub label: Option<String>,
    pub size: Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub view_formats: Option<Vec<TextureFormat>>,
}

impl TextureDescriptorCreateInfo {
    pub fn d2(
        label: Option<String>,
        width: u32,
        height: u32,
        format: Option<TextureFormat>,
    ) -> TextureDescriptorCreateInfo {
        TextureDescriptorCreateInfo {
            label,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: format.unwrap_or(TextureFormat::Rgba8Unorm),
            usage: TextureUsages::all(),
            view_formats: None,
        }
    }

    pub fn get(&self) -> wgpu::TextureDescriptor {
        wgpu::TextureDescriptor {
            label: match &self.label {
                Some(label) => Some(&label),
                None => None,
            },
            size: self.size,
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension,
            format: self.format,
            usage: self.usage,
            view_formats: match &self.view_formats {
                Some(view_formats) => view_formats.as_slice(),
                None => &[],
            },
        }
    }
}

#[derive(Clone)]
pub struct CreateBuffer {
    pub handle: BufferHandle,
    pub buffer_create_info: BufferCreateInfo,
}

#[derive(Clone)]
pub struct CreateSampler {
    pub handle: SamplerHandle,
    pub sampler_descriptor: SamplerDescriptor<'static>,
}

#[derive(Clone)]
pub struct CreateMaterialRenderPipeline {
    pub handle: MaterialRenderPipelineHandle,
    pub shader_code: String,
}

#[derive(Clone)]
pub struct BufferCreateInfo {
    pub label: Option<String>,
    pub contents: Vec<u8>,
    pub usage: wgpu::BufferUsages,
}

#[derive(Clone)]
pub struct UpdateBuffer {
    pub handle: BufferHandle,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct InitTextureData {
    pub data: Vec<u8>,
    pub data_layout: wgpu::ImageDataLayout,
}

#[derive(Clone, Debug)]
pub enum EBindingResource {
    Texture(TextureHandle),
    Constants(BufferHandle),
    Sampler(SamplerHandle),
}

#[derive(Clone)]
pub struct VirtualPassSet {
    pub vertex_buffers: Vec<BufferHandle>,
    pub binding_resources: Vec<Vec<EBindingResource>>,
}

#[derive(Clone)]
pub struct DrawObject {
    pub id: u32,
    pub vertex_buffers: Vec<BufferHandle>,
    pub vertex_count: u32,
    pub index_buffer: Option<BufferHandle>,
    pub index_count: Option<u32>,
    pub binding_resources: Vec<Vec<EBindingResource>>,
    pub virtual_pass_set: Option<VirtualPassSet>,
    pub render_pipeline: String,
}

#[derive(Clone)]
pub struct ResizeInfo {
    pub window_id: isize,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub struct UpdateTexture {
    pub handle: TextureHandle,
    pub texture_data: InitTextureData,
    pub size: Extent3d,
}

#[derive(Clone)]
pub struct CreateTexture {
    pub handle: TextureHandle,
    pub texture_descriptor_create_info: TextureDescriptorCreateInfo,
    pub init_data: Option<InitTextureData>,
}

#[derive(Clone)]
pub struct CreateVirtualTexture {
    pub handle: TextureHandle,
    pub source: Arc<Mutex<Box<dyn TVirtualTextureSource>>>,
}

#[derive(Clone)]
pub struct CreateVirtualTexturePass {
    pub key: VirtualTexturePassKey,
    pub surface_size: glam::UVec2,
    pub settings: VirtualTextureSetting,
}

#[derive(Debug, Clone, Hash, PartialEq, Copy, Eq)]
pub struct VirtualTexturePassKey {
    pub physical_texture_handle: TextureHandle,
    pub page_table_texture_handle: TextureHandle,
}

#[derive(Debug, Clone, Hash, PartialEq, Copy, Eq)]
pub struct IBLTexturesKey {
    pub brdflut_texture: TextureHandle,
    pub pre_filter_cube_map_texture: TextureHandle,
    pub irradiance_texture: TextureHandle,
}

#[derive(Clone)]
pub struct VirtualTexturePassResize {
    pub key: VirtualTexturePassKey,
    pub surface_size: glam::UVec2,
}

#[derive(Clone)]
pub struct CreateUITexture {
    pub handle: EGUITextureHandle,
    pub referencing_texture_handle: TextureHandle,
}

#[derive(Clone)]
pub struct CreateIBLBake {
    pub key: IBLTexturesKey,
    pub file_path: PathBuf,
    pub bake_info: BakeInfo,
    pub save_dir: Option<PathBuf>,
}

#[derive(Clone)]
pub struct UploadPrebakeIBL {
    pub key: IBLTexturesKey,
    pub brdf_data: Vec<u8>,
    pub pre_filter_data: Vec<u8>,
    pub irradiance_data: Vec<u8>,
}

pub trait RenderTask {
    fn exec(&mut self);
}

pub type TaskType = Arc<Mutex<Box<dyn FnMut(&mut crate::renderer::Renderer) + Send>>>;

#[derive(Clone)]
pub struct PresentInfo {
    pub window_id: isize,
    pub draw_objects: Vec<DrawObject>,
    pub virtual_texture_pass: Option<VirtualTexturePassKey>,
}

#[derive(Clone)]
pub enum RenderCommand {
    CreateIBLBake(CreateIBLBake),
    CreateTexture(CreateTexture),
    CreateUITexture(CreateUITexture),
    CreateBuffer(CreateBuffer),
    UpdateBuffer(UpdateBuffer),
    UpdateTexture(UpdateTexture),
    UiOutput(EGUIRenderOutput),
    Resize(ResizeInfo),
    CreateVirtualTextureSource(CreateVirtualTexture),
    CreateVirtualTexturePass(CreateVirtualTexturePass),
    VirtualTexturePassResize(VirtualTexturePassResize),
    ClearVirtualTexturePass(VirtualTexturePassKey),
    Task(TaskType),
    Settings(RenderSettings),
    Present(PresentInfo),
    RemoveWindow(isize),
    ChangeViewMode(EViewModeType),
    CreateSampler(CreateSampler),
    CreateMaterialRenderPipeline(CreateMaterialRenderPipeline),
    UploadPrebakeIBL(UploadPrebakeIBL),
    #[cfg(feature = "renderdoc")]
    CaptureFrame,
}

impl RenderCommand {
    pub fn create_task(
        task: impl FnMut(&mut crate::renderer::Renderer) + Send + 'static,
    ) -> RenderCommand {
        RenderCommand::Task(Arc::new(Mutex::new(Box::new(task))))
    }
}

#[derive(Clone, Default)]
pub struct RenderOutput {
    pub create_texture_handles: HashSet<TextureHandle>,
    pub create_buffer_handles: HashSet<BufferHandle>,
    pub create_ibl_handles: HashSet<IBLTexturesKey>,
}

pub enum ERenderOutputType {
    CreateIBLBake(TextureHandle),
    CreateTexture(TextureHandle),
    CreateUITexture(EGUITextureHandle),
    CreateBuffer(BufferHandle),
    UpdateBuffer(BufferHandle),
    UpdateTexture(TextureHandle),
    DrawObject(u32),
    UiOutput(isize),
    Resize(isize),
    CreateVirtualTextureSource(TextureHandle),
    Present(isize),
    RemoveWindow(isize),
    CreateSampler(SamplerHandle),
    CreateMaterialRenderPipeline(MaterialRenderPipelineHandle),
    UploadPrebakeIBL(TextureHandle),
}

pub struct RenderOutput2 {
    pub ty: ERenderOutputType,
    pub error: Option<RwLock<crate::error::Error>>,
}
