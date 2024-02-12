use crate::{bake_info::BakeInfo, egui_render::EGUIRenderOutput};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use wgpu::*;

pub type BufferHandle = u64;
pub type EGUITextureHandle = u64;
pub type TextureHandle = u64;

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
pub struct PBRMaterial {
    pub albedo_texture: Option<TextureHandle>,
    pub normal_texture: Option<TextureHandle>,
    pub metallic_texture: Option<TextureHandle>,
    pub roughness_texture: Option<TextureHandle>,
    pub ibl_texture: Option<TextureHandle>,
}

#[derive(Clone)]
pub struct PhongMaterial {
    pub constants: crate::render_pipeline::phong_pipeline::Constants,
    pub diffuse_texture: Option<TextureHandle>,
    pub specular_texture: Option<TextureHandle>,
}

#[derive(Clone)]
pub enum EMaterialType {
    Phong(PhongMaterial),
    PBR(PBRMaterial),
}

#[derive(Clone)]
pub struct DrawObject {
    pub vertex_buffers: Vec<BufferHandle>,
    pub vertex_count: u32,
    pub index_buffer: Option<BufferHandle>,
    pub index_count: Option<u32>,
    pub material_type: EMaterialType,
}

#[derive(Clone)]
pub struct ResizeInfo {
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
pub struct CreateUITexture {
    pub handle: EGUITextureHandle,
    pub referencing_texture_handle: TextureHandle,
}

#[derive(Clone)]
pub struct CreateIBLBake {
    pub handle: TextureHandle,
    pub file_path: PathBuf,
    pub bake_info: BakeInfo,
}

pub trait RenderTask {
    fn exec(&mut self);
}

pub type TaskType = Arc<Mutex<Box<dyn FnMut(&mut crate::renderer::Renderer) + Send>>>;

#[derive(Clone)]
pub enum RenderCommand {
    CreateIBLBake(CreateIBLBake),
    CreateTexture(CreateTexture),
    CreateUITexture(CreateUITexture),
    CreateBuffer(CreateBuffer),
    UpdateBuffer(UpdateBuffer),
    UpdateTexture(UpdateTexture),
    DrawObject(DrawObject),
    UiOutput(EGUIRenderOutput),
    Resize(ResizeInfo),
    Task(TaskType),
    Present,
    #[cfg(feature = "renderdoc")]
    CaptureFrame,
}

#[derive(Clone, Default)]
pub struct RenderOutput {
    pub create_texture_handles: HashSet<TextureHandle>,
    pub create_buffer_handles: HashSet<BufferHandle>,
    pub create_ibl_handles: HashSet<TextureHandle>,
}
