use crate::{
    camera::Camera,
    default_textures::DefaultTextures,
    egui_context::{DataSource, DrawImage, EGUIContext, EGUIContextRenderer},
    file_manager::FileManager,
    gizmo::FGizmo,
    logger::Logger,
    native_window::NativeWindow,
    render_pipeline::attachment_pipeline::AttachmentPipeline,
    shader::shader_library::ShaderLibrary,
    thread_pool::{SingleConsumeChnnel, SyncWait, ThreadPool},
    util::{self, change_working_directory, get_resource_path},
    wgpu_context::WGPUContext,
};
use egui_wgpu_backend::ScreenDescriptor;
use rs_media::{
    audio_format::EAudioSampleType, audio_player_item::AudioPlayerItem,
    video_frame_player::VideoFramePlayer,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};
use wgpu::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use winit::event::{Event::*, VirtualKeyCode};
use winit::event_loop::ControlFlow;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct TextureHandle(u32);

impl TextureHandle {
    pub fn new(id: u32) -> Arc<TextureHandle> {
        Arc::new(TextureHandle(id))
    }

    pub fn is_need_release(handle: &Arc<TextureHandle>) -> bool {
        Arc::strong_count(handle) <= 1
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct EGUITextureHandle(u32);

impl EGUITextureHandle {
    pub fn new(id: u32) -> Arc<EGUITextureHandle> {
        Arc::new(EGUITextureHandle(id))
    }

    pub fn is_need_release(handle: &Arc<EGUITextureHandle>) -> bool {
        Arc::strong_count(handle) <= 1
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
    pub fn d2(label: Option<String>, width: u32, height: u32) -> TextureDescriptorCreateInfo {
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
            format: TextureFormat::Rgba8Unorm,
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

pub struct CreateTexture {
    pub handle: Arc<TextureHandle>,
    pub texture_descriptor_create_info: TextureDescriptorCreateInfo,
}

pub struct CreateUITexture {
    pub handle: Arc<EGUITextureHandle>,
    pub referencing_texture_handle: Arc<TextureHandle>,
}

pub struct UpdateTexture {
    pub handle: Arc<TextureHandle>,
    data: Vec<u8>,
    data_layout: wgpu::ImageDataLayout,
    size: Extent3d,
}

pub struct CreateResourceRequestInfo {
    pub create_textures: Vec<CreateTexture>,
    pub create_ui_textures: Vec<CreateUITexture>,
}

pub struct UpdateResourceRequestInfo {
    pub update_textures: Vec<UpdateTexture>,
}

pub enum RenderMessage {
    CreateResource(CreateResourceRequestInfo),
    UpdateResource(UpdateResourceRequestInfo),
    Resized(winit::dpi::PhysicalSize<u32>),
    Redraw(egui::FullOutput),
}

pub struct RenderOutputMessage {
    textures: HashSet<Arc<TextureHandle>>,
    texture_handles: HashMap<Arc<EGUITextureHandle>, egui::TextureId>,
}

pub struct RenderContext {
    wgpu_context: WGPUContext,
    screen_descriptor: ScreenDescriptor,
    egui_context_renderer: EGUIContextRenderer,
    attachment_pipeline: AttachmentPipeline,
    textures: HashMap<Arc<TextureHandle>, wgpu::Texture>,
    texture_handles: HashMap<Arc<EGUITextureHandle>, egui::TextureId>,
}

impl RenderContext {
    pub fn new(
        wgpu_context: WGPUContext,
        screen_descriptor: ScreenDescriptor,
        platform_context: egui::Context,
    ) -> RenderContext {
        DefaultTextures::default()
            .lock()
            .unwrap()
            .init(&wgpu_context.device, &wgpu_context.queue);
        ShaderLibrary::default().lock().unwrap().load_shader_from(
            &wgpu_context.device,
            &FileManager::default().lock().unwrap().get_shader_dir_path(),
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

        RenderContext {
            wgpu_context,
            screen_descriptor,
            egui_context_renderer,
            attachment_pipeline,
            textures: HashMap::new(),
            texture_handles: HashMap::new(),
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
                store: true,
            },
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: true,
            }),
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: true,
            }),
        );
    }

    fn clear_textures(&mut self) {
        self.textures
            .retain(|key, _| TextureHandle::is_need_release(key) == false);
    }

    fn create_textures(&mut self, tasks: Vec<CreateTexture>) -> HashSet<Arc<TextureHandle>> {
        let mut new_textures = HashSet::new();
        for create_texture in tasks {
            let texture = self
                .wgpu_context
                .device
                .create_texture(&create_texture.texture_descriptor_create_info.get());
            self.textures.insert(create_texture.handle.clone(), texture);
            new_textures.insert(create_texture.handle);
        }
        new_textures
    }

    fn process_create_requested_info(
        &mut self,
        redraw_requested_info: CreateResourceRequestInfo,
    ) -> Option<RenderOutputMessage> {
        let mut render_output_message = RenderOutputMessage {
            texture_handles: HashMap::new(),
            textures: HashSet::new(),
        };

        render_output_message.textures =
            self.create_textures(redraw_requested_info.create_textures);

        let mut old_datas: Vec<egui::TextureId> = Vec::new();
        for create_ui_texture in redraw_requested_info.create_ui_textures {
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

        let mut retain_handles: HashMap<Arc<EGUITextureHandle>, egui::TextureId> = HashMap::new();
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

        Some(render_output_message)
    }

    pub fn tick(&mut self, render_message: RenderMessage) -> Option<RenderOutputMessage> {
        self.clear_textures();

        match render_message {
            RenderMessage::Resized(size) => {
                self.screen_descriptor.physical_height = size.height;
                self.screen_descriptor.physical_width = size.width;
                self.wgpu_context.window_resized(size);
                None
            }
            RenderMessage::CreateResource(create_resource_request_info) => {
                self.process_create_requested_info(create_resource_request_info)
            }
            RenderMessage::Redraw(full_output) => {
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
                debug_assert!(full_output.textures_delta.free.is_empty());
                self.egui_context_renderer.render(
                    &full_output,
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
                                &update_texture.data,
                                update_texture.data_layout,
                                update_texture.size,
                            );
                        }
                        None => {}
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
                backends: wgpu::Backends::VULKAN,
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            }),
        );

        ThreadPool::render().lock().unwrap().spawn({
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

        ThreadPool::audio().lock().unwrap().spawn({
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
                sync_wait.accept_stop();
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
        let camera = Camera::default(
            screen_descriptor.physical_width,
            screen_descriptor.physical_height,
        );
        let mut data_source = DataSource::new(camera);

        let texture_handle = TextureHandle::new(0);
        let egui_texture_handle = EGUITextureHandle::new(0);
        let mut avaliable_texture_handles: HashMap<Arc<EGUITextureHandle>, egui::TextureId> =
            HashMap::new();
        let create_texture = CreateTexture {
            handle: texture_handle.clone(),
            texture_descriptor_create_info: TextureDescriptorCreateInfo::d2(
                Some(String::from("video")),
                1280,
                720,
            ),
        };
        let create_ui_texture = CreateUITexture {
            handle: egui_texture_handle.clone(),
            referencing_texture_handle: create_texture.handle.clone(),
        };
        render_thread_channel.to_b(RenderMessage::CreateResource(CreateResourceRequestInfo {
            create_textures: vec![create_texture],
            create_ui_textures: vec![create_ui_texture],
        }));

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
                                    _ => {}
                                }
                            }
                        }
                        winit::event::WindowEvent::Resized(size) => {
                            render_thread_channel.to_b(RenderMessage::Resized(*size));
                        }
                        _ => {}
                    },
                    DeviceEvent { .. } => {}
                    UserEvent(_) => {}
                    Suspended => {}
                    Resumed => {}
                    MainEventsCleared => {}
                    RedrawRequested(_) => {
                        let elapsed = std::time::Instant::now() - self.current_frame_start_time;
                        Self::sync_fps(elapsed, data_source.target_fps, control_flow);
                        self.current_frame_start_time = std::time::Instant::now();

                        video_frame_player.tick();
                        if let Some(video_frame) = video_frame_player.get_current_frame() {
                            let info = UpdateResourceRequestInfo {
                                update_textures: vec![UpdateTexture {
                                    handle: texture_handle.clone(),
                                    data: video_frame.image.to_vec(),
                                    data_layout: wgpu::ImageDataLayout {
                                        offset: 0,
                                        bytes_per_row: Some(1280 * 4),
                                        rows_per_image: None,
                                    },
                                    size: Extent3d {
                                        width: 1280,
                                        height: 720,
                                        depth_or_array_layers: 1,
                                    },
                                }],
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
                                    size: egui::Vec2::new(1280.0, 720.0),
                                })
                            }
                            None => {}
                        }

                        let full_output = self.egui_context.layout(&mut data_source);
                        render_thread_channel.to_b(RenderMessage::Redraw(full_output));
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
