use crate::{
    bake_info::BakeInfo, egui_render::EGUIRenderOutput, renderer::EPipelineType,
    scene_viewport::SceneViewport, view_mode::EViewModeType,
    virtual_texture_source::TVirtualTextureSource,
};
use rs_core_minimal::settings::{RenderSettings, VirtualTextureSetting};
use rs_render_types::MaterialOptions;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};
use wgpu::*;

pub type BufferHandle = u64;
pub type EGUITextureHandle = u64;
pub type TextureHandle = u64;
pub type SamplerHandle = u64;
pub type MaterialRenderPipelineHandle = u64;

#[derive(Debug, Clone, Copy)]
pub struct FrameBufferOptions {
    pub color: TextureHandle,
    pub depth: TextureHandle,
}

#[derive(Debug, Clone, Copy)]
pub enum ERenderTargetType {
    SurfaceTexture(isize),
    FrameBuffer(FrameBufferOptions),
}

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

    pub fn d3(
        label: Option<String>,
        width: u32,
        height: u32,
        depth: u32,
        format: Option<TextureFormat>,
    ) -> TextureDescriptorCreateInfo {
        TextureDescriptorCreateInfo {
            label,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: format.unwrap_or(TextureFormat::Rgba8Unorm),
            usage: {
                let mut usage = TextureUsages::all();
                usage.remove(TextureUsages::RENDER_ATTACHMENT);
                usage
            },
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
    pub shader_code: HashMap<MaterialOptions, String>,
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
    // TODO: Optimization of object with large memory usage
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
pub struct MultiDrawIndirect {
    pub indirect_buffer_handle: BufferHandle,
    pub indirect_offset: wgpu::BufferAddress,
    pub count: u32,
}

#[derive(Clone)]
pub struct Draw {
    pub instances: std::ops::Range<u32>,
}

#[derive(Clone)]
pub enum EDrawCallType {
    MultiDrawIndirect(MultiDrawIndirect),
    Draw(Draw),
}

#[derive(Clone)]
pub struct ShadowMapping {
    pub is_skin: bool,
    pub vertex_buffers: Vec<BufferHandle>,
    pub binding_resources: Vec<Vec<EBindingResource>>,
}

#[derive(Clone)]
pub struct Viewport {
    pub rect: glam::Vec4,
    pub depth_range: std::ops::Range<f32>,
}

#[derive(Clone)]
pub struct DrawObject {
    pub id: u32,
    pub vertex_buffers: Vec<BufferHandle>,
    pub vertex_count: u32,

    pub pipeline: EPipelineType,
    pub index_buffer: Option<BufferHandle>,
    pub index_count: Option<u32>,
    pub binding_resources: Vec<Vec<EBindingResource>>,
    pub virtual_pass_set: Option<VirtualPassSet>,
    pub draw_call_type: EDrawCallType,
    pub shadow_mapping: Option<ShadowMapping>,
    pub scissor_rect: Option<glam::UVec4>,
    pub viewport: Option<Viewport>,
    pub debug_group_label: Option<String>,
}

impl DrawObject {
    pub fn new(
        id: u32,
        vertex_buffers: Vec<BufferHandle>,
        vertex_count: u32,
        pipeline: EPipelineType,
        index_buffer: Option<BufferHandle>,
        index_count: Option<u32>,
        binding_resources: Vec<Vec<EBindingResource>>,
    ) -> DrawObject {
        DrawObject {
            id,
            vertex_buffers,
            vertex_count,
            pipeline,
            index_buffer,
            index_count,
            binding_resources,
            virtual_pass_set: None,
            shadow_mapping: None,
            scissor_rect: None,
            viewport: None,
            draw_call_type: EDrawCallType::Draw(Draw { instances: 0..1 }),
            debug_group_label: None,
        }
    }
}

#[derive(Clone)]
pub struct ResizeInfo {
    pub window_id: isize,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub struct ScaleChangedInfo {
    pub window_id: isize,
    pub new_factor: f32,
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
pub struct SceneLight {
    pub point_light_shapes: Vec<crate::constants::Sphere3D>,
    pub frustum: crate::global_uniform::CameraFrustum,
    pub cluster_lights_placeholder: BufferHandle,
    pub cluster_light_indices_placeholder: BufferHandle,
}

#[derive(Clone)]
pub struct PresentInfo {
    pub render_target_type: ERenderTargetType,
    pub draw_objects: Vec<DrawObject>,
    pub virtual_texture_pass: Option<VirtualTexturePassKey>,
    pub scene_viewport: SceneViewport,
    pub depth_texture_handle: Option<TextureHandle>,
    pub scene_light: Option<SceneLight>,
}

impl PresentInfo {
    pub fn new(
        render_target_type: ERenderTargetType,
        draw_objects: Vec<DrawObject>,
    ) -> PresentInfo {
        PresentInfo {
            render_target_type,
            draw_objects,
            virtual_texture_pass: None,
            scene_viewport: SceneViewport::new(),
            depth_texture_handle: None,
            scene_light: None,
        }
    }
}

#[derive(Clone)]
pub struct ClearDepthTexture {
    pub handle: TextureHandle,
}

#[derive(Clone)]
pub struct BuiltinShaderChanged {
    pub name: String,
    pub source: String,
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
    ScaleChanged(ScaleChangedInfo),
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
    CreateDefaultIBL(IBLTexturesKey),
    ClearDepthTexture(ClearDepthTexture),
    BuiltinShaderChanged(BuiltinShaderChanged),
    DestroyTextures(Vec<TextureHandle>),
    #[cfg(feature = "renderdoc")]
    CaptureFrame,
    WindowRedrawRequestedBegin(isize),
    WindowRedrawRequestedEnd(isize),
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
