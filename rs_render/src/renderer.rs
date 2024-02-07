use crate::command::*;
use crate::default_textures::DefaultTextures;
use crate::depth_texture::DepthTexture;
use crate::error::Result;
use crate::gpu_vertex_buffer::GpuVertexBufferImp;
use crate::render_pipeline::attachment_pipeline::AttachmentPipeline;
use crate::render_pipeline::phong_pipeline::PhongPipeline;
use crate::shader_library::ShaderLibrary;
use crate::{egui_render::EGUIRenderer, wgpu_context::WGPUContext};
use std::collections::{HashMap, VecDeque};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

pub struct Renderer {
    wgpu_context: WGPUContext,
    gui_renderer: EGUIRenderer,
    screen_descriptor: egui_wgpu::ScreenDescriptor,
    shader_library: ShaderLibrary,
    create_iblbake_commands: Vec<CreateIBLBake>,
    create_texture_commands: Vec<CreateTexture>,
    create_uitexture_commands: Vec<CreateUITexture>,
    create_buffer_commands: Vec<CreateBuffer>,
    update_buffer_commands: Vec<UpdateBuffer>,
    update_texture_commands: Vec<UpdateTexture>,
    draw_object_commands: Vec<DrawObject>,
    ui_output_commands: VecDeque<crate::egui_render::EGUIRenderOutput>,
    resize_commands: VecDeque<ResizeInfo>,

    textures: HashMap<u64, Texture>,
    buffers: HashMap<u64, Buffer>,
    ui_textures: HashMap<u64, egui::TextureId>,

    phong_pipeline: PhongPipeline,
    attachment_pipeline: AttachmentPipeline,

    depth_texture: DepthTexture,
    default_textures: DefaultTextures,

    texture_descriptors: HashMap<u64, TextureDescriptorCreateInfo>,
    buffer_infos: HashMap<u64, BufferCreateInfo>,
}

impl Renderer {
    pub fn from_context(
        wgpu_context: WGPUContext,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
    ) -> Renderer {
        let egui_render_pass = EGUIRenderer::new(
            wgpu_context.get_device(),
            wgpu_context.get_current_swapchain_format(),
            1,
        );
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [surface_width, surface_height],
            pixels_per_point: scale_factor,
        };
        let mut shader_library = ShaderLibrary::new();
        shader_library.load_inner_shader(wgpu_context.get_device());

        let phong_pipeline = PhongPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &wgpu_context.get_current_swapchain_format(),
            false,
        );
        let depth_texture =
            DepthTexture::new(surface_width, surface_height, wgpu_context.get_device());
        let default_textures =
            DefaultTextures::new(wgpu_context.get_device(), wgpu_context.get_queue());
        let attachment_pipeline = AttachmentPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &wgpu_context.get_current_swapchain_format(),
        );

        Renderer {
            wgpu_context,
            gui_renderer: egui_render_pass,
            screen_descriptor,
            shader_library,
            create_iblbake_commands: Vec::new(),
            create_texture_commands: Vec::new(),
            create_uitexture_commands: Vec::new(),
            create_buffer_commands: Vec::new(),
            update_buffer_commands: Vec::new(),
            update_texture_commands: Vec::new(),
            draw_object_commands: Vec::new(),
            ui_output_commands: VecDeque::new(),
            resize_commands: VecDeque::new(),
            textures: HashMap::new(),
            buffers: HashMap::new(),
            ui_textures: HashMap::new(),
            phong_pipeline,
            attachment_pipeline,
            depth_texture,
            default_textures,
            texture_descriptors: HashMap::new(),
            buffer_infos: HashMap::new(),
        }
    }

    pub fn from_window<W>(
        window: &W,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
    ) -> Result<Renderer>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let wgpu_context = WGPUContext::new(
            window,
            surface_width,
            surface_height,
            Some(wgpu::PowerPreference::HighPerformance),
            Some(wgpu::InstanceDescriptor {
                #[cfg(target_os = "windows")]
                backends: wgpu::Backends::DX12,
                #[cfg(not(target_os = "windows"))]
                backends: wgpu::Backends::PRIMARY,
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
                flags: wgpu::InstanceFlags::default(),
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            }),
        );
        let wgpu_context = match wgpu_context {
            Ok(wgpu_context) => wgpu_context,
            Err(err) => return Err(err),
        };
        Ok(Self::from_context(
            wgpu_context,
            surface_width,
            surface_height,
            scale_factor,
        ))
    }

    pub fn set_new_window<W>(
        &mut self,
        window: &W,
        surface_width: u32,
        surface_height: u32,
    ) -> Result<()>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        self.wgpu_context
            .set_new_window(window, surface_width, surface_height)
    }

    pub fn present(&mut self) -> Option<RenderOutput> {
        while let Some(resize_command) = self.resize_commands.pop_front() {
            if resize_command.width <= 0 || resize_command.height <= 0 {
                continue;
            }
            self.screen_descriptor.size_in_pixels[0] = resize_command.width;
            self.screen_descriptor.size_in_pixels[1] = resize_command.height;
            self.wgpu_context
                .window_resized(resize_command.width, resize_command.height);
            let device = self.wgpu_context.get_device();
            self.depth_texture =
                DepthTexture::new(resize_command.width, resize_command.height, device);
        }

        let mut render_output = RenderOutput::default();
        let device = self.wgpu_context.get_device();
        let queue = self.wgpu_context.get_queue();
        for create_texture_command in &self.create_texture_commands {
            let texture =
                device.create_texture(&create_texture_command.texture_descriptor_create_info.get());
            if let Some(init_data) = &create_texture_command.init_data {
                queue.write_texture(
                    texture.as_image_copy(),
                    &init_data.data,
                    init_data.data_layout,
                    create_texture_command.texture_descriptor_create_info.size,
                );
            }
            let handle = create_texture_command.handle;
            render_output.create_texture_handles.insert(handle);
            self.textures.insert(handle, texture);
            self.texture_descriptors.insert(
                handle,
                create_texture_command
                    .texture_descriptor_create_info
                    .clone(),
            );
        }
        self.create_texture_commands.clear();

        for create_buffer_command in &self.create_buffer_commands {
            let descriptor = BufferInitDescriptor {
                label: create_buffer_command.buffer_create_info.label.as_deref(),
                contents: &create_buffer_command.buffer_create_info.contents,
                usage: create_buffer_command.buffer_create_info.usage,
            };
            let new_buffer = device.create_buffer_init(&descriptor);
            let handle = create_buffer_command.handle;
            render_output.create_buffer_handles.insert(handle);
            self.buffers.insert(handle, new_buffer);
            self.buffer_infos
                .insert(handle, create_buffer_command.buffer_create_info.clone());
        }
        self.create_buffer_commands.clear();

        for update_buffer_command in &self.update_buffer_commands {
            if let Some(buffer) = self.buffers.get(&update_buffer_command.handle) {
                let (sender, receiver) = std::sync::mpsc::channel();
                buffer.slice(..).map_async(wgpu::MapMode::Write, {
                    move |result| {
                        sender.send(result).unwrap();
                    }
                });
                device.poll(wgpu::Maintain::Wait);
                if let Ok(Ok(_)) = receiver.recv() {
                    let mut padded_buffer_view = buffer.slice(..).get_mapped_range_mut();
                    let padded_buffer = padded_buffer_view.as_mut();
                    padded_buffer.copy_from_slice(&update_buffer_command.data);
                    drop(padded_buffer_view);
                }
                buffer.unmap();
            }
        }
        self.update_buffer_commands.clear();

        for update_texture_command in &self.update_texture_commands {
            if let Some(texture) = self.textures.get(&update_texture_command.handle) {
                queue.write_texture(
                    texture.as_image_copy(),
                    &update_texture_command.texture_data.data,
                    update_texture_command.texture_data.data_layout,
                    update_texture_command.size,
                );
            }
        }
        self.update_texture_commands.clear();

        for create_uitexture_command in &self.create_uitexture_commands {
            if let Some(texture) = self
                .textures
                .get(&create_uitexture_command.referencing_texture_handle)
            {
                let ui_texture_id = self.gui_renderer.create_image2(
                    device,
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    None,
                );
                self.ui_textures
                    .insert(create_uitexture_command.handle, ui_texture_id);
            }
        }
        self.create_uitexture_commands.clear();

        let texture = match self.wgpu_context.get_current_surface_texture() {
            Ok(texture) => texture,
            Err(err) => {
                if err != wgpu::SurfaceError::Outdated {
                    log::warn!("{}", err);
                }
                return None;
            }
        };

        let output_view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = &self.depth_texture.get_view();
        self.clear_buffer(&output_view, depth_texture_view);
        self.draw_objects(&output_view);
        self.draw_object_commands.clear();

        while let Some(output) = self.ui_output_commands.pop_front() {
            self.gui_renderer
                .render(output, queue, device, &self.screen_descriptor, &output_view)
        }
        texture.present();
        return Some(render_output);
    }

    fn clear_buffer(&self, surface_texture_view: &TextureView, depth_texture_view: &TextureView) {
        self.attachment_pipeline.draw(
            self.wgpu_context.get_device(),
            self.wgpu_context.get_queue(),
            surface_texture_view,
            depth_texture_view,
            Operations {
                load: LoadOp::Clear(Color::TRANSPARENT),
                store: StoreOp::Store,
            },
            Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            Some(Operations {
                load: LoadOp::Clear(0),
                store: StoreOp::Store,
            }),
        );
    }

    pub fn load_shader<K>(&mut self, shaders: HashMap<K, String>)
    where
        K: ToString,
    {
        self.shader_library
            .load_shader_from(shaders, self.wgpu_context.get_device());
    }

    pub fn send_command(&mut self, command: RenderCommand) -> Option<RenderOutput> {
        match command {
            RenderCommand::CreateIBLBake(command) => self.create_iblbake_commands.push(command),
            RenderCommand::CreateTexture(command) => self.create_texture_commands.push(command),
            RenderCommand::CreateUITexture(command) => self.create_uitexture_commands.push(command),
            RenderCommand::CreateBuffer(command) => self.create_buffer_commands.push(command),
            RenderCommand::UpdateBuffer(command) => self.update_buffer_commands.push(command),
            RenderCommand::UpdateTexture(command) => self.update_texture_commands.push(command),
            RenderCommand::DrawObject(command) => self.draw_object_commands.push(command),
            RenderCommand::UiOutput(command) => self.ui_output_commands.push_back(command),
            RenderCommand::Resize(command) => self.resize_commands.push_back(command),
            RenderCommand::Present => {
                return self.present();
            }
        }
        return None;
    }

    fn draw_objects(&self, surface_texture_view: &wgpu::TextureView) {
        let device = self.wgpu_context.get_device();
        let queue = self.wgpu_context.get_queue();
        for draw_object_command in &self.draw_object_commands {
            match &draw_object_command.material_type {
                EMaterialType::Phong(material) => {
                    let diffuse_texture: &Texture;
                    let specular_texture: &Texture;
                    let binding = self.default_textures.get_white_texture();
                    if let Some(diffuse_texture_handle) = material.diffuse_texture {
                        diffuse_texture = self
                            .textures
                            .get(&diffuse_texture_handle)
                            .unwrap_or(&binding);
                    } else {
                        diffuse_texture = &binding;
                    }
                    if let Some(specular_texture_handle) = material.specular_texture {
                        specular_texture = self
                            .textures
                            .get(&specular_texture_handle)
                            .unwrap_or(&binding);
                    } else {
                        specular_texture = &binding;
                    }

                    let mut vertex_buffers = Vec::<&Buffer>::new();
                    let mut index_buffer: Option<&wgpu::Buffer> = None;

                    for vertex_buffer in &draw_object_command.vertex_buffers {
                        if let Some(vertex_buffer) = self.buffers.get(&vertex_buffer) {
                            vertex_buffers.push(vertex_buffer);
                        }
                    }
                    if vertex_buffers.is_empty() {
                        return;
                    }
                    if let Some(handle) = draw_object_command.index_buffer {
                        if let Some(buffer) = self.buffers.get(&handle) {
                            index_buffer = Some(buffer);
                        }
                    }
                    let mesh_buffer = GpuVertexBufferImp {
                        vertex_buffers: &vertex_buffers,
                        vertex_count: draw_object_command.vertex_count,
                        index_buffer,
                        index_count: draw_object_command.index_count,
                    };
                    let diffuse_texture_view =
                        diffuse_texture.create_view(&TextureViewDescriptor::default());
                    let specular_texture_view =
                        specular_texture.create_view(&TextureViewDescriptor::default());
                    self.phong_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &self.depth_texture.get_view(),
                        &material.constants,
                        &[mesh_buffer],
                        &diffuse_texture_view,
                        &specular_texture_view,
                    );
                }
                EMaterialType::PBR(_) => todo!(),
            }
        }
    }
}
