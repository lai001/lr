use crate::{
    acceleration_bake::AccelerationBaker,
    bake_info::BakeInfo,
    brigde_data::gpu_vertex_buffer::GpuVertexBufferImp,
    camera::{Camera, CameraInputEventHandle, DefaultCameraInputEventHandle},
    default_textures::DefaultTextures,
    egui_context::{DataSource, DrawImage, EGUIContext, EGUIContextRenderer},
    file_manager::FileManager,
    light::{DirectionalLight, PointLight, SpotLight},
    logger::Logger,
    model_loader,
    native_window::NativeWindow,
    render_pipeline::{
        attachment_pipeline::AttachmentPipeline, next_pbr_pipeline::NextPBRPipeline,
        next_phong_pipeline::NextPhongPipeline,
    },
    resource_manager::ResourceManager,
    shader::shader_library::ShaderLibrary,
    thread_pool::{SingleConsumeChnnel, SyncWait, ThreadPool},
    util::{self, change_working_directory, get_resource_path},
    wgpu_context::WGPUContext,
};
use egui_wgpu_backend::ScreenDescriptor;
use rs_foundation::id_generator::IDGenerator;
use rs_media::{
    audio_format::EAudioSampleType, audio_player_item::AudioPlayerItem,
    video_frame_player::VideoFramePlayer,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, Extent3d, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::event::{Event::*, VirtualKeyCode};
use winit::event_loop::ControlFlow;

lazy_static! {
    static ref GLOBAL_TEXTURE_IDGENERATOR: Mutex<IDGenerator> = Mutex::new(IDGenerator::new());
    static ref GLOBAL_BUFFER_IDGENERATOR: Mutex<IDGenerator> = Mutex::new(IDGenerator::new());
    static ref GLOBAL_GUI_TEXTURE_IDGENERATOR: Mutex<IDGenerator> = Mutex::new(IDGenerator::new());
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct TextureHandle {
    id: Arc<u64>,
}

impl TextureHandle {
    pub fn next() -> TextureHandle {
        let new_id = GLOBAL_TEXTURE_IDGENERATOR.lock().unwrap().get_next_id();
        TextureHandle {
            id: Arc::new(new_id),
        }
    }

    pub fn is_need_release(&self) -> bool {
        Arc::strong_count(&self.id) <= 1
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct EGUITextureHandle {
    id: Arc<u64>,
}

impl EGUITextureHandle {
    pub fn next() -> EGUITextureHandle {
        let new_id = GLOBAL_GUI_TEXTURE_IDGENERATOR.lock().unwrap().get_next_id();
        EGUITextureHandle {
            id: Arc::new(new_id),
        }
    }

    pub fn is_need_release(&self) -> bool {
        Arc::strong_count(&self.id) <= 1
    }
}

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

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct BufferHandle {
    id: Arc<u64>,
}

impl BufferHandle {
    pub fn next() -> BufferHandle {
        let new_id = GLOBAL_BUFFER_IDGENERATOR.lock().unwrap().get_next_id();
        BufferHandle {
            id: Arc::new(new_id),
        }
    }

    pub fn is_need_release(&self) -> bool {
        Arc::strong_count(&self.id) <= 1
    }
}

pub struct CreateBuffer {
    pub handle: BufferHandle,
    pub buffer_create_info: BufferCreateInfo,
}

pub struct BufferCreateInfo {
    pub label: Option<String>,
    pub contents: Vec<u8>,
    pub usage: wgpu::BufferUsages,
}

pub struct UpdateBuffer {
    pub handle: BufferHandle,
    data: Vec<u8>,
}

pub struct InitTextureData {
    pub data: Vec<u8>,
    pub data_layout: wgpu::ImageDataLayout,
}

pub struct CreateTexture {
    pub handle: TextureHandle,
    pub texture_descriptor_create_info: TextureDescriptorCreateInfo,
    pub init_data: Option<InitTextureData>,
}

pub struct CreateIBLBake {
    pub handle: TextureHandle,
    pub file_path: String,
    pub bake_info: crate::bake_info::BakeInfo,
}

pub struct CreateUITexture {
    pub handle: EGUITextureHandle,
    pub referencing_texture_handle: TextureHandle,
}

pub struct UpdateTexture {
    pub handle: TextureHandle,
    texture_data: InitTextureData,
    size: Extent3d,
}

pub enum ECreateResourceRequestType {
    Texture(Vec<CreateTexture>),
    UITexture(Vec<CreateUITexture>),
    Buffer(Vec<CreateBuffer>),
    Bake(Vec<CreateIBLBake>),
}

pub struct UpdateResourceRequestInfo {
    pub update_textures: Vec<UpdateTexture>,
    pub update_buffers: Vec<UpdateBuffer>,
}

#[derive(Clone)]
pub struct PBRMaterial {
    pub constants: crate::render_pipeline::next_pbr_pipeline::Constants,
    pub albedo_texture: Option<TextureHandle>,
    pub normal_texture: Option<TextureHandle>,
    pub metallic_texture: Option<TextureHandle>,
    pub roughness_texture: Option<TextureHandle>,
    pub ibl_texture: Option<TextureHandle>,
}

#[derive(Clone)]
pub struct PhongMaterial {
    pub constants: crate::render_pipeline::next_phong_pipeline::Constants,
    pub diffuse_texture: Arc<TextureHandle>,
    pub specular_texture: Arc<TextureHandle>,
}

#[derive(Clone)]
pub enum EMaterialType {
    Phong(PhongMaterial),
    PBR(PBRMaterial),
}

#[derive(Clone)]
pub struct DrawObjectTask {
    pub vertex_buffers: Vec<BufferHandle>,
    pub vertex_count: u32,
    pub index_buffer: Option<BufferHandle>,
    pub index_count: Option<u32>,
    pub material_type: EMaterialType,
}

pub struct RedrawInfo {
    pub ui_output: egui::FullOutput,
    pub draw_object_tasks: Vec<DrawObjectTask>,
}

pub enum RenderMessage {
    CreateResource(Vec<ECreateResourceRequestType>),
    UpdateResource(UpdateResourceRequestInfo),
    Resized(winit::dpi::PhysicalSize<u32>),
    Redraw(RedrawInfo),
}

pub struct RenderOutputMessage {
    textures: HashSet<TextureHandle>,
    texture_handles: HashMap<EGUITextureHandle, egui::TextureId>,
    buffers: HashSet<BufferHandle>,
}

pub struct RenderContext {
    wgpu_context: WGPUContext,
    screen_descriptor: ScreenDescriptor,
    egui_context_renderer: EGUIContextRenderer,
    attachment_pipeline: AttachmentPipeline,
    textures: HashMap<TextureHandle, wgpu::Texture>,
    texture_handles: HashMap<EGUITextureHandle, egui::TextureId>,
    buffers: HashMap<BufferHandle, wgpu::Buffer>,
    ibl_textures: HashMap<TextureHandle, AccelerationBaker>,
    next_phong_pipeline: NextPhongPipeline,
    pbr_pipeline: NextPBRPipeline,
    default_textures: DefaultTextures,
    new_window_size: Option<winit::dpi::PhysicalSize<u32>>,
}

impl RenderContext {
    pub fn new(
        wgpu_context: WGPUContext,
        screen_descriptor: ScreenDescriptor,
        platform_context: egui::Context,
    ) -> RenderContext {
        ShaderLibrary::default().lock().unwrap().load_shader_from(
            &wgpu_context.device,
            &FileManager::default().get_shader_dir_path(),
        );
        let egui_context_renderer = EGUIContextRenderer::new(
            platform_context,
            &wgpu_context.device,
            wgpu_context.get_current_swapchain_format(),
            1,
        );
        let attachment_pipeline = AttachmentPipeline::new(
            &wgpu_context.device,
            &wgpu_context.get_current_swapchain_format(),
        );

        let next_phong_pipeline = NextPhongPipeline::new(
            &wgpu_context.device,
            &wgpu_context.get_current_swapchain_format(),
            false,
        );
        let pbr_pipeline = NextPBRPipeline::new(
            &wgpu_context.device,
            &wgpu_context.get_current_swapchain_format(),
            false,
        );
        let mut default_textures = DefaultTextures::new();
        default_textures.init(&wgpu_context.device, &wgpu_context.queue);
        RenderContext {
            wgpu_context,
            screen_descriptor,
            egui_context_renderer,
            attachment_pipeline,
            textures: HashMap::new(),
            texture_handles: HashMap::new(),
            buffers: HashMap::new(),
            next_phong_pipeline,
            pbr_pipeline,
            default_textures,
            ibl_textures: HashMap::new(),
            new_window_size: None,
        }
    }

    fn clear_buffer(&self, surface_texture_view: &wgpu::TextureView) {
        self.attachment_pipeline.draw(
            &self.wgpu_context.device,
            &self.wgpu_context.queue,
            surface_texture_view,
            &self.wgpu_context.get_depth_texture_view(),
            wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: wgpu::StoreOp::Store,
            }),
        );
    }

    fn clear_textures(&mut self) {
        self.textures
            .retain(|key, _| TextureHandle::is_need_release(key) == false);
    }

    fn clear_buffers(&mut self) {
        self.buffers
            .retain(|key, _| BufferHandle::is_need_release(key) == false);
    }

    fn create_textures(&mut self, tasks: Vec<CreateTexture>) -> HashSet<TextureHandle> {
        let mut new_textures = HashSet::new();
        for create_texture in tasks {
            let texture = self
                .wgpu_context
                .device
                .create_texture(&create_texture.texture_descriptor_create_info.get());
            if let Some(init_data) = create_texture.init_data {
                self.wgpu_context.queue.write_texture(
                    texture.as_image_copy(),
                    &init_data.data,
                    init_data.data_layout,
                    create_texture.texture_descriptor_create_info.size,
                );
            }
            self.textures.insert(create_texture.handle.clone(), texture);
            new_textures.insert(create_texture.handle);
        }
        new_textures
    }

    fn create_buffers(&mut self, tasks: Vec<CreateBuffer>) -> HashSet<BufferHandle> {
        let mut new_buffers = HashSet::new();
        for create_buffer in tasks {
            let descriptor = BufferInitDescriptor {
                label: match &create_buffer.buffer_create_info.label {
                    Some(label) => Some(&label),
                    None => None,
                },
                contents: &create_buffer.buffer_create_info.contents,
                usage: create_buffer.buffer_create_info.usage,
            };
            let new_buffer = self.wgpu_context.device.create_buffer_init(&descriptor);
            self.buffers
                .insert(create_buffer.handle.clone(), new_buffer);
            new_buffers.insert(create_buffer.handle);
        }
        new_buffers
    }

    fn process_create_requested_info(
        &mut self,
        create_resource_types: Vec<ECreateResourceRequestType>,
    ) -> Option<RenderOutputMessage> {
        let mut render_output_message = RenderOutputMessage {
            textures: HashSet::new(),
            texture_handles: HashMap::new(),
            buffers: HashSet::new(),
        };

        for create_resource_type in create_resource_types {
            match create_resource_type {
                ECreateResourceRequestType::Texture(create_textures) => {
                    render_output_message.textures = self.create_textures(create_textures);
                }
                ECreateResourceRequestType::Buffer(create_buffers) => {
                    render_output_message.buffers = self.create_buffers(create_buffers);
                }
                ECreateResourceRequestType::UITexture(create_ui_textures) => {
                    let mut old_datas: Vec<egui::TextureId> = Vec::new();
                    for create_ui_texture in create_ui_textures {
                        let texture = self
                            .textures
                            .get(&create_ui_texture.referencing_texture_handle)
                            .unwrap();

                        let texture_id = self.egui_context_renderer.create_image2(
                            &self.wgpu_context.device,
                            &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                            None,
                        );
                        let handle = create_ui_texture.handle;
                        match self.texture_handles.insert(handle.clone(), texture_id) {
                            Some(old_id) => {
                                old_datas.push(old_id);
                            }
                            None => {}
                        }

                        render_output_message
                            .texture_handles
                            .insert(handle.clone(), texture_id);
                    }

                    let mut retain_handles: HashMap<EGUITextureHandle, egui::TextureId> =
                        HashMap::new();
                    for (texture_handle, texture_id) in &self.texture_handles {
                        if EGUITextureHandle::is_need_release(&texture_handle) {
                            old_datas.push(*texture_id);
                        } else {
                            retain_handles.insert(texture_handle.clone(), *texture_id);
                        }
                    }
                    self.texture_handles = retain_handles;
                    self.egui_context_renderer.remove_texture_ids(&old_datas);
                    old_datas.clear();
                }
                ECreateResourceRequestType::Bake(create_ibl_bakes) => {
                    for create_ibl_bake in create_ibl_bakes {
                        let mut baker = AccelerationBaker::new(
                            &self.wgpu_context.device,
                            &self.wgpu_context.queue,
                            &create_ibl_bake.file_path,
                            create_ibl_bake.bake_info,
                        );
                        baker.bake(&self.wgpu_context.device, &self.wgpu_context.queue);
                        self.ibl_textures.insert(create_ibl_bake.handle, baker);
                    }
                }
            }
        }
        Some(render_output_message)
    }

    fn draw_objects(
        &self,
        surface_texture_view: &wgpu::TextureView,
        draw_object_tasks: Vec<DrawObjectTask>,
    ) {
        for draw_object_task in draw_object_tasks {
            match draw_object_task.material_type {
                EMaterialType::Phong(material) => {
                    if let (Some(diffuse_texture), Some(specular_texture)) = (
                        self.textures.get(&material.diffuse_texture),
                        self.textures.get(&material.specular_texture),
                    ) {
                        let mut vertex_buffers = Vec::<&Buffer>::new();
                        let mut index_buffer: Option<&wgpu::Buffer> = None;

                        for vertex_buffer in draw_object_task.vertex_buffers {
                            if let Some(vertex_buffer) = self.buffers.get(&vertex_buffer) {
                                vertex_buffers.push(vertex_buffer);
                            }
                        }
                        if let Some(handle) = draw_object_task.index_buffer {
                            if let Some(buffer) = self.buffers.get(&handle) {
                                index_buffer = Some(buffer);
                            }
                        }
                        let mesh_buffer = GpuVertexBufferImp {
                            vertex_buffers: &vertex_buffers,
                            vertex_count: draw_object_task.vertex_count,
                            index_buffer,
                            index_count: draw_object_task.index_count,
                        };
                        let diffuse_texture_view =
                            diffuse_texture.create_view(&TextureViewDescriptor::default());
                        let specular_texture_view =
                            specular_texture.create_view(&TextureViewDescriptor::default());
                        self.next_phong_pipeline.draw(
                            &self.wgpu_context.device,
                            &self.wgpu_context.queue,
                            surface_texture_view,
                            &self.wgpu_context.get_depth_texture_view(),
                            &material.constants,
                            &[mesh_buffer],
                            &diffuse_texture_view,
                            &specular_texture_view,
                        );
                    }
                }
                EMaterialType::PBR(material) => {
                    if let Some(handle) = material.ibl_texture {
                        if let Some(ibl_baker) = self.ibl_textures.get(&handle) {
                            let mut vertex_buffers = Vec::<&Buffer>::new();
                            let mut index_buffer: Option<&wgpu::Buffer> = None;

                            for vertex_buffer in draw_object_task.vertex_buffers {
                                if let Some(vertex_buffer) = self.buffers.get(&vertex_buffer) {
                                    vertex_buffers.push(vertex_buffer);
                                }
                            }
                            if let Some(handle) = draw_object_task.index_buffer {
                                if let Some(buffer) = self.buffers.get(&handle) {
                                    index_buffer = Some(buffer);
                                }
                            }
                            let mesh_buffer = GpuVertexBufferImp {
                                vertex_buffers: &vertex_buffers,
                                vertex_count: draw_object_task.vertex_count,
                                index_buffer,
                                index_count: draw_object_task.index_count,
                            };
                            let albedo_texture_view =
                                self.get_texture_view_fallback(material.albedo_texture);
                            let normal_texture_view =
                                self.get_normal_texture_view_fallback(material.normal_texture);
                            let metallic_texture_view =
                                self.get_texture_view_fallback(material.metallic_texture);
                            let roughness_texture_view =
                                self.get_texture_view_fallback(material.roughness_texture);
                            self.pbr_pipeline.draw(
                                &self.wgpu_context.device,
                                &self.wgpu_context.queue,
                                surface_texture_view,
                                &self.wgpu_context.get_depth_texture_view(),
                                &material.constants,
                                &[mesh_buffer],
                                &crate::render_pipeline::next_pbr_pipeline::Material {
                                    albedo_texture_view,
                                    normal_texture_view,
                                    metallic_texture_view,
                                    roughness_texture_view,
                                    brdflut_texture_view: ibl_baker.get_brdflut_texture_view(),
                                    pre_filter_cube_map_texture_view: ibl_baker
                                        .get_pre_filter_cube_map_texture_view(),
                                    irradiance_texture_view: ibl_baker
                                        .get_irradiance_texture_view(),
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn tick(&mut self, render_message: RenderMessage) -> Option<RenderOutputMessage> {
        self.clear_textures();
        self.clear_buffers();

        match render_message {
            RenderMessage::Resized(size) => {
                self.new_window_size = Some(size);
                // self.screen_descriptor.physical_height = size.height;
                // self.screen_descriptor.physical_width = size.width;
                // self.wgpu_context.window_resized(size);
                None
            }
            RenderMessage::CreateResource(create_resource_request_info) => {
                self.process_create_requested_info(create_resource_request_info)
            }
            RenderMessage::Redraw(redraw_info) => {
                if let Some(size) = self.new_window_size {
                    self.screen_descriptor.physical_height = size.height;
                    self.screen_descriptor.physical_width = size.width;
                    self.wgpu_context.window_resized(size);
                    self.new_window_size = None;
                }
                let result = self.wgpu_context.get_current_surface_texture();
                if let Err(error) = result {
                    log::warn!("{error}");
                    return None;
                }
                let surface = result.unwrap();
                let surface_texture_view = surface
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.clear_buffer(&surface_texture_view);
                debug_assert!(redraw_info.ui_output.textures_delta.free.is_empty());
                self.draw_objects(&surface_texture_view, redraw_info.draw_object_tasks);
                self.egui_context_renderer.render(
                    &redraw_info.ui_output,
                    &self.wgpu_context.queue,
                    &self.wgpu_context.device,
                    &self.screen_descriptor,
                    &surface_texture_view,
                );
                surface.present();
                None
            }
            RenderMessage::UpdateResource(update_resource_request_info) => {
                for update_texture in update_resource_request_info.update_textures {
                    match self.textures.get(&update_texture.handle) {
                        Some(target_texture) => {
                            self.wgpu_context.queue.write_texture(
                                target_texture.as_image_copy(),
                                &update_texture.texture_data.data,
                                update_texture.texture_data.data_layout,
                                update_texture.size,
                            );
                        }
                        None => {}
                    }
                }
                for update_buffer in update_resource_request_info.update_buffers {
                    if let Some(target_buffer) = self.buffers.get(&update_buffer.handle) {
                        let (sender, receiver) = std::sync::mpsc::channel();
                        target_buffer.slice(..).map_async(wgpu::MapMode::Write, {
                            move |result| {
                                sender.send(result).unwrap();
                            }
                        });
                        self.wgpu_context.device.poll(wgpu::Maintain::Wait);
                        if let Ok(Ok(_)) = receiver.recv() {
                            let mut padded_buffer_view =
                                target_buffer.slice(..).get_mapped_range_mut();
                            let padded_buffer = padded_buffer_view.as_mut();
                            padded_buffer.copy_from_slice(&update_buffer.data);
                            drop(padded_buffer_view);
                        }
                        target_buffer.unmap();
                    }
                }
                None
            }
        }
    }

    fn spawn_render_thread(
        screen_descriptor: ScreenDescriptor,
        window: &winit::window::Window,
        platform_context: egui::Context,
    ) -> Arc<SingleConsumeChnnel<RenderMessage, RenderOutputMessage>> {
        let channel =
            SingleConsumeChnnel::<RenderMessage, RenderOutputMessage>::shared(Some(2), None);

        let wgpu_context = WGPUContext::new(
            &window,
            Some(wgpu::PowerPreference::HighPerformance),
            Some(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
                flags: wgpu::InstanceFlags::default(),
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            }),
        );

        ThreadPool::render().spawn({
            let channel = channel.clone();
            move || {
                let mut render_context =
                    RenderContext::new(wgpu_context, screen_descriptor, platform_context);
                channel.from_a_block_current_thread(|render_message| {
                    match render_context.tick(render_message) {
                        Some(render_output_message) => {
                            channel.to_a(render_output_message);
                        }
                        None => {}
                    }
                });
            }
        });
        channel
    }

    fn get_texture_view_fallback(
        &self,
        texture_handle: Option<TextureHandle>,
    ) -> wgpu::TextureView {
        if let Some(texture_handle) = texture_handle {
            if let Some(texture) = self.textures.get(&texture_handle) {
                texture.create_view(&TextureViewDescriptor::default())
            } else {
                self.default_textures.get_white_texture_view()
            }
        } else {
            self.default_textures.get_white_texture_view()
        }
    }

    fn get_normal_texture_view_fallback(
        &self,
        texture_handle: Option<TextureHandle>,
    ) -> wgpu::TextureView {
        if let Some(texture_handle) = texture_handle {
            if let Some(texture) = self.textures.get(&texture_handle) {
                texture.create_view(&TextureViewDescriptor::default())
            } else {
                self.default_textures.get_normal_texture_view()
            }
        } else {
            self.default_textures.get_normal_texture_view()
        }
    }
}

pub struct ExampleApplication {
    native_window: NativeWindow,
    egui_context: EGUIContext,
    current_frame_start_time: std::time::Instant,
}

impl ExampleApplication {
    pub fn new() -> ExampleApplication {
        let native_window = NativeWindow::new();
        let screen_descriptor = Self::get_screen_descriptor(&native_window.window);
        let egui_context = EGUIContext::new(&screen_descriptor);
        ExampleApplication {
            native_window,
            egui_context,
            current_frame_start_time: std::time::Instant::now(),
        }
    }

    fn spawn_audio_thread() -> Arc<SyncWait> {
        let sync_wait = SyncWait::shared();
        ThreadPool::audio().spawn({
            let sync_wait = sync_wait.clone();
            move || {
                let mut audio_device = rs_media::audio_device::AudioDevice::new();
                audio_device.play();
                let mut audio_player_item =
                    AudioPlayerItem::new(&util::get_resource_path("Remote/BigBuckBunny.mp4"));
                let mut data: VecDeque<f32> = VecDeque::with_capacity(1024 * 8);

                loop {
                    if sync_wait.is_stop() {
                        break;
                    }
                    while audio_device.get_buffer_len() < 1024 * 8 {
                        match audio_player_item.try_recv() {
                            Ok(frame) => {
                                let pcm_buffer = &frame.pcm_buffer;
                                let format = pcm_buffer.get_audio_format();
                                debug_assert_eq!(format.channels_per_frame, 2);
                                debug_assert_eq!(
                                    format.get_sample_type(),
                                    EAudioSampleType::Float32
                                );
                                debug_assert_eq!(format.is_non_interleaved(), true);

                                let channel_data_0: &[f32] = pcm_buffer.get_channel_data_view(0);
                                let channel_data_1: &[f32] = pcm_buffer.get_channel_data_view(1);
                                data.clear();
                                for (first, second) in
                                    channel_data_0.iter().zip(channel_data_1.iter())
                                {
                                    data.push_back(*first);
                                    data.push_back(*second);
                                }
                                audio_device.push_buffer(data.make_contiguous());
                            }
                            Err(error) => match error {
                                rs_media::error::Error::EndOfFile => break,
                                rs_media::error::Error::TryAgain => {
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                }
                                rs_media::error::Error::Disconnected => break,
                            },
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                log::trace!("Thread exit.");
                sync_wait.finish();
            }
        });
        sync_wait
    }

    pub fn run(mut self) {
        change_working_directory();
        let logger = Logger::new();
        rs_media::init();

        let video_file_path = get_resource_path("Remote/BigBuckBunny.mp4");
        let mut video_frame_player = VideoFramePlayer::new(&video_file_path);
        video_frame_player.start();

        let render_thread_channel = RenderContext::spawn_render_thread(
            Self::get_screen_descriptor(&self.native_window.window),
            &self.native_window.window,
            self.egui_context.get_platform_context(),
        );

        let audio_sync_wait = Self::spawn_audio_thread();

        let screen_descriptor = Self::get_screen_descriptor(&self.native_window.window);
        let mut camera = Camera::default(
            screen_descriptor.physical_width,
            screen_descriptor.physical_height,
        );
        let mut data_source = DataSource::new(camera);

        let texture_handle = TextureHandle::next();
        let egui_texture_handle = EGUITextureHandle::next();
        let mut avaliable_texture_handles: HashMap<EGUITextureHandle, egui::TextureId> =
            HashMap::new();
        let create_texture = CreateTexture {
            handle: texture_handle.clone(),
            texture_descriptor_create_info: TextureDescriptorCreateInfo::d2(
                Some(String::from("video")),
                1280,
                720,
                None,
            ),
            init_data: None,
        };
        let create_ui_texture = CreateUITexture {
            handle: egui_texture_handle.clone(),
            referencing_texture_handle: create_texture.handle.clone(),
        };

        render_thread_channel.to_b(RenderMessage::CreateResource(vec![
            ECreateResourceRequestType::Texture(vec![create_texture]),
            ECreateResourceRequestType::UITexture(vec![create_ui_texture]),
        ]));

        let ibl_texture_handle = TextureHandle::next();
        let mut draw_object_tasks: Vec<DrawObjectTask> = Vec::new();

        {
            render_thread_channel.to_b(RenderMessage::CreateResource(vec![
                ECreateResourceRequestType::Bake(vec![CreateIBLBake {
                    handle: ibl_texture_handle.clone(),
                    file_path: get_resource_path("Remote/neon_photostudio_2k.exr"),
                    bake_info: BakeInfo {
                        is_bake_environment: true,
                        is_bake_irradiance: true,
                        is_bake_brdflut: true,
                        is_bake_pre_filter: true,
                        environment_cube_map_length: 512,
                        irradiance_cube_map_length: 32,
                        irradiance_sample_count: 4096,
                        pre_filter_cube_map_length: 2048,
                        pre_filter_cube_map_max_mipmap_level: 5,
                        pre_filter_sample_count: 1024,
                        brdflutmap_length: 256,
                        brdf_sample_count: 4096,
                        is_read_back: false,
                    },
                }]),
            ]));

            let clusters = model_loader::ModelLoader::load_from_file2(
                &FileManager::default().get_resource_path("Remote/Monkey.fbx"),
            );

            for cluster in clusters {
                let vertex_buffer_handle = BufferHandle::next();
                let index_buffer_handle = BufferHandle::next();
                let mut texture_handles: HashMap<russimp::texture::TextureType, TextureHandle> =
                    HashMap::new();
                let vertex_buffer_create_info = BufferCreateInfo {
                    label: Some(String::from("vertex_buffer")),
                    contents: rs_foundation::cast_to_raw_buffer(&cluster.vertex_buffer).to_vec(),
                    usage: wgpu::BufferUsages::VERTEX,
                };
                let index_buffer_create_info = BufferCreateInfo {
                    label: Some(String::from("index_buffer")),
                    contents: rs_foundation::cast_to_raw_buffer(&cluster.index_buffer).to_vec(),
                    usage: wgpu::BufferUsages::INDEX,
                };
                let mut create_textures: Vec<CreateTexture> = Vec::new();
                for (texture_type, image_path) in cluster.textures_dic {
                    let texture_handle = TextureHandle::next();
                    texture_handles.insert(texture_type.clone(), texture_handle.clone());
                    let image = ResourceManager::default()
                        .get_cache_image(&image_path)
                        .unwrap();
                    let image = image.to_rgba8();
                    let create_texture = CreateTexture {
                        handle: texture_handle,
                        texture_descriptor_create_info: TextureDescriptorCreateInfo::d2(
                            Some(String::from(format!("{:?}", texture_type))),
                            image.width(),
                            image.height(),
                            None,
                        ),
                        init_data: Some(InitTextureData {
                            data: image.to_vec(),
                            data_layout: wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(image.width() * 4),
                                rows_per_image: None,
                            },
                        }),
                    };
                    create_textures.push(create_texture);
                }
                render_thread_channel.to_b(RenderMessage::CreateResource(vec![
                    ECreateResourceRequestType::Buffer(vec![
                        CreateBuffer {
                            handle: vertex_buffer_handle.clone(),
                            buffer_create_info: vertex_buffer_create_info,
                        },
                        CreateBuffer {
                            handle: index_buffer_handle.clone(),
                            buffer_create_info: index_buffer_create_info,
                        },
                    ]),
                    ECreateResourceRequestType::Texture(create_textures),
                ]));

                let pbr_material = PBRMaterial {
                    constants: crate::render_pipeline::next_pbr_pipeline::Constants::new(
                        DirectionalLight::default(),
                        PointLight::default(),
                        SpotLight::default(),
                        glam::Mat4::IDENTITY,
                        camera.get_view_matrix(),
                        camera.get_projection_matrix(),
                        camera.get_world_location(),
                        0.0,
                        0.0,
                    ),
                    albedo_texture: texture_handles
                        .get(&russimp::texture::TextureType::Diffuse)
                        .cloned(),
                    normal_texture: texture_handles
                        .get(&russimp::texture::TextureType::Normals)
                        .cloned(),
                    metallic_texture: texture_handles
                        .get(&russimp::texture::TextureType::Metalness)
                        .cloned(),
                    roughness_texture: texture_handles
                        .get(&russimp::texture::TextureType::Shininess)
                        .cloned(),
                    ibl_texture: Some(ibl_texture_handle.clone()),
                };
                let task = DrawObjectTask {
                    vertex_buffers: vec![vertex_buffer_handle.clone()],
                    vertex_count: cluster.vertex_buffer.len() as u32,
                    index_buffer: Some(index_buffer_handle.clone()),
                    index_count: Some(cluster.index_buffer.len() as u32),
                    material_type: EMaterialType::PBR(pbr_material),
                };

                draw_object_tasks.push(task);
            }
        }

        let mut is_cursor_visible = true;
        let mut virtual_key_code_state_map =
            std::collections::HashMap::<VirtualKeyCode, winit::event::ElementState>::new();

        let event_loop = self.native_window.event_loop;
        event_loop.run({
            move |event, _, control_flow| {
                self.egui_context.handle_event(&event);

                match &event {
                    NewEvents(_) => {}
                    WindowEvent { event, .. } => match event {
                        winit::event::WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        winit::event::WindowEvent::KeyboardInput { input, .. } => {
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                match virtual_keycode {
                                    VirtualKeyCode::Escape => {
                                        *control_flow = ControlFlow::Exit;
                                    }
                                    VirtualKeyCode::F1 => {
                                        if input.state == winit::event::ElementState::Released {
                                            is_cursor_visible = !is_cursor_visible;
                                            self.native_window
                                                .window
                                                .set_cursor_visible(is_cursor_visible);
                                            if is_cursor_visible {
                                                self.native_window
                                                    .window
                                                    .set_cursor_grab(
                                                        winit::window::CursorGrabMode::None,
                                                    )
                                                    .unwrap();
                                            } else {
                                                self.native_window
                                                    .window
                                                    .set_cursor_grab(
                                                        winit::window::CursorGrabMode::Confined,
                                                    )
                                                    .unwrap();
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if let Some(keycode) = input.virtual_keycode {
                                virtual_key_code_state_map.insert(keycode, input.state);
                            }
                        }
                        winit::event::WindowEvent::Resized(size) => {
                            log::trace!("Window Resized. {size:?}");
                            render_thread_channel.to_b(RenderMessage::Resized(*size));
                        }
                        _ => {}
                    },
                    DeviceEvent { event, .. } => match event {
                        winit::event::DeviceEvent::MouseMotion { delta } => {
                            DefaultCameraInputEventHandle::mouse_motion_handle(
                                &mut camera,
                                *delta,
                                is_cursor_visible,
                                data_source.motion_speed,
                            );
                        }
                        _ => {}
                    },
                    UserEvent(_) => {}
                    Suspended => {}
                    Resumed => {}
                    MainEventsCleared => {}
                    RedrawRequested(_) => {
                        for (virtual_key_code, element_state) in &virtual_key_code_state_map {
                            DefaultCameraInputEventHandle::keyboard_input_handle(
                                &mut camera,
                                virtual_key_code,
                                element_state,
                                is_cursor_visible,
                                data_source.movement_speed,
                            );
                        }

                        let elapsed = std::time::Instant::now() - self.current_frame_start_time;
                        Self::sync_fps(elapsed, data_source.target_fps, control_flow);
                        self.current_frame_start_time = std::time::Instant::now();

                        video_frame_player.tick();
                        if let Some(video_frame) = video_frame_player.get_current_frame() {
                            let info = UpdateResourceRequestInfo {
                                update_textures: vec![UpdateTexture {
                                    handle: texture_handle.clone(),
                                    texture_data: InitTextureData {
                                        data: video_frame.image.to_vec(),
                                        data_layout: wgpu::ImageDataLayout {
                                            offset: 0,
                                            bytes_per_row: Some(1280 * 4),
                                            rows_per_image: None,
                                        },
                                    },
                                    size: Extent3d {
                                        width: 1280,
                                        height: 720,
                                        depth_or_array_layers: 1,
                                    },
                                }],
                                update_buffers: vec![],
                            };
                            render_thread_channel.to_b(RenderMessage::UpdateResource(info))
                        }

                        {
                            while let Ok(render_output_message) =
                                render_thread_channel.from_b_try_recv()
                            {
                                for (k, v) in render_output_message.texture_handles {
                                    avaliable_texture_handles.insert(k, v);
                                }
                            }
                        }

                        match avaliable_texture_handles.get(&egui_texture_handle) {
                            Some(texture_id) => {
                                data_source.video_frame = Some(DrawImage {
                                    texture_id: *texture_id,
                                    size: egui::Vec2::new(1280.0 / 2.0, 720.0 / 2.0),
                                })
                            }
                            None => {}
                        }

                        for draw_object_task in &mut draw_object_tasks {
                            match &mut draw_object_task.material_type {
                                EMaterialType::Phong(_) => {}
                                EMaterialType::PBR(material) => {
                                    material.constants.projection = camera.get_projection_matrix();
                                    material.constants.view = camera.get_view_matrix();
                                    material.constants.view_position = camera.get_world_location();
                                    material.constants.metalness_factor =
                                        data_source.metalness_factor;
                                    material.constants.roughness_factor =
                                        data_source.roughness_factor;
                                }
                            }
                        }

                        let redraw_info = RedrawInfo {
                            ui_output: self.egui_context.layout(&mut data_source),
                            draw_object_tasks: draw_object_tasks.clone(),
                        };
                        render_thread_channel.to_b(RenderMessage::Redraw(redraw_info));
                    }
                    RedrawEventsCleared => {
                        self.native_window.window.request_redraw();
                    }
                    LoopDestroyed => {
                        render_thread_channel.send_stop_signal_and_wait();
                        audio_sync_wait.send_stop_signal_and_wait();
                        logger.flush();
                    }
                }
            }
        });
    }

    pub fn get_screen_descriptor(
        window: &winit::window::Window,
    ) -> egui_wgpu_backend::ScreenDescriptor {
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: window.inner_size().width,
            physical_height: window.inner_size().height,
            scale_factor: window.scale_factor() as f32,
        };
        screen_descriptor
    }

    fn sync_fps(
        elapsed: std::time::Duration,
        fps: u64,
        control_flow: &mut winit::event_loop::ControlFlow,
    ) {
        let fps = std::time::Duration::from_secs_f32(1.0 / fps as f32);
        let wait: std::time::Duration;
        if fps < elapsed {
            wait = std::time::Duration::from_millis(0);
        } else {
            wait = fps - elapsed;
        }
        let new_inst = std::time::Instant::now() + wait;
        *control_flow = winit::event_loop::ControlFlow::WaitUntil(new_inst);
    }
}
