use crate::acceleration_bake::AccelerationBaker;
use crate::cube_map::CubeMap;
use crate::default_textures::DefaultTextures;
use crate::depth_texture::DepthTexture;
use crate::error::Result;
use crate::gpu_vertex_buffer::GpuVertexBufferImp;
use crate::render_pipeline::attachment_pipeline::AttachmentPipeline;
use crate::render_pipeline::shading::{self, ShadingPipeline};
use crate::sampler_cache::SamplerCache;
use crate::shader_library::ShaderLibrary;
use crate::virtual_texture_pass::VirtualTexturePass;
use crate::virtual_texture_source::VirtualTextureSource;
use crate::{command::*, ibl_readback};
use crate::{egui_render::EGUIRenderer, wgpu_context::WGPUContext};
use image::{GenericImage, GenericImageView};
use rs_core_minimal::settings::{self, RenderSettings};
use std::collections::{HashMap, VecDeque};
use std::ops::Deref;
use std::path::Path;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

pub struct Renderer {
    wgpu_context: WGPUContext,
    gui_renderer: EGUIRenderer,
    screen_descriptor: egui_wgpu::ScreenDescriptor,
    shader_library: ShaderLibrary,
    create_iblbake_commands: VecDeque<CreateIBLBake>,
    create_texture_commands: Vec<CreateTexture>,
    create_uitexture_commands: Vec<CreateUITexture>,
    create_buffer_commands: Vec<CreateBuffer>,
    update_buffer_commands: Vec<UpdateBuffer>,
    update_texture_commands: Vec<UpdateTexture>,
    draw_object_commands: Vec<DrawObject>,
    ui_output_commands: VecDeque<crate::egui_render::EGUIRenderOutput>,
    resize_commands: VecDeque<ResizeInfo>,
    task_commands: VecDeque<TaskType>,

    textures: HashMap<u64, Texture>,
    buffers: HashMap<u64, Buffer>,
    ui_textures: HashMap<u64, egui::TextureId>,
    ibl_bakes: HashMap<u64, AccelerationBaker>,

    shading_pipeline: ShadingPipeline,
    attachment_pipeline: AttachmentPipeline,

    depth_texture: DepthTexture,
    default_textures: DefaultTextures,

    texture_descriptors: HashMap<u64, TextureDescriptorCreateInfo>,
    buffer_infos: HashMap<u64, BufferCreateInfo>,

    #[cfg(feature = "renderdoc")]
    render_doc_context: Option<crate::renderdoc::Context>,

    virtual_texture_pass: Option<VirtualTexturePass>,

    settings: RenderSettings,
}

impl Renderer {
    pub fn from_context(
        wgpu_context: WGPUContext,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
        shaders: HashMap<String, String>,
        settings: RenderSettings,
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
        shader_library.load_shader_from(shaders, wgpu_context.get_device());
        let mut sampler_cache = SamplerCache::new();

        let shading_pipeline = ShadingPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &wgpu_context.get_current_swapchain_format(),
            false,
            &mut sampler_cache,
        );
        let depth_texture = DepthTexture::new(
            surface_width,
            surface_height,
            wgpu_context.get_device(),
            Some("Base.DepthTexture"),
        );
        let default_textures =
            DefaultTextures::new(wgpu_context.get_device(), wgpu_context.get_queue());
        let attachment_pipeline = AttachmentPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &wgpu_context.get_current_swapchain_format(),
        );

        let virtual_texture_pass: Option<VirtualTexturePass>;
        if settings.virtual_texture_setting.is_enable {
            virtual_texture_pass = VirtualTexturePass::new(
                wgpu_context.get_device(),
                &shader_library,
                false,
                glam::uvec2(surface_width, surface_height),
                settings.virtual_texture_setting.clone(),
            )
            .ok();
        } else {
            virtual_texture_pass = None;
        }

        Renderer {
            wgpu_context,
            gui_renderer: egui_render_pass,
            screen_descriptor,
            shader_library,
            create_iblbake_commands: VecDeque::new(),
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
            shading_pipeline,
            attachment_pipeline,
            depth_texture,
            default_textures,
            texture_descriptors: HashMap::new(),
            buffer_infos: HashMap::new(),
            task_commands: VecDeque::new(),
            ibl_bakes: HashMap::new(),
            #[cfg(feature = "renderdoc")]
            render_doc_context: crate::renderdoc::Context::new().ok(),
            virtual_texture_pass,
            settings,
        }
    }

    pub fn from_window<W>(
        window: &W,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
        shaders: HashMap<String, String>,
        settings: RenderSettings,
    ) -> Result<Renderer>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let wgpu_context = WGPUContext::new(
            window,
            surface_width,
            surface_height,
            Some(match settings.power_preference {
                settings::PowerPreference::None => PowerPreference::None,
                settings::PowerPreference::LowPower => PowerPreference::LowPower,
                settings::PowerPreference::HighPerformance => PowerPreference::HighPerformance,
            }),
            Some(wgpu::InstanceDescriptor {
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
                flags: wgpu::InstanceFlags::default(),
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
                backends: match settings.backends {
                    settings::Backends::Primary => Backends::PRIMARY,
                    settings::Backends::Vulkan => Backends::VULKAN,
                    settings::Backends::GL => Backends::GL,
                    settings::Backends::DX12 => Backends::DX12,
                },
            }),
        )?;
        Ok(Self::from_context(
            wgpu_context,
            surface_width,
            surface_height,
            scale_factor,
            shaders,
            settings,
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
        #[cfg(feature = "renderdoc")]
        let mut is_capture_frame = false;
        #[cfg(feature = "renderdoc")]
        {
            if let Some(render_doc_context) = &mut self.render_doc_context {
                if render_doc_context.capture_commands.is_empty() == false {
                    is_capture_frame = true;
                }
                render_doc_context.capture_commands.clear();
            }
            if is_capture_frame {
                if let Some(render_doc_context) = &mut self.render_doc_context {
                    render_doc_context.start_capture(self.wgpu_context.get_device());
                }
            }
        }

        while let Some(resize_command) = self.resize_commands.pop_front() {
            if resize_command.width <= 0 || resize_command.height <= 0 {
                continue;
            }
            self.surface_size_will_change(glam::uvec2(resize_command.width, resize_command.height));
        }

        while let Some(task_command) = self.task_commands.pop_front() {
            let mut task = task_command.lock().unwrap();
            task(self);
        }

        let mut render_output = RenderOutput::default();

        for create_texture_command in &self.create_texture_commands {
            let device = self.wgpu_context.get_device();
            let queue = self.wgpu_context.get_queue();
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
            let device = self.wgpu_context.get_device();
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
            let device = self.wgpu_context.get_device();
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
            let queue = self.wgpu_context.get_queue();
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
            let device = self.wgpu_context.get_device();
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

        while let Some(create_iblbake_command) = self.create_iblbake_commands.pop_front() {
            let device = self.wgpu_context.get_device();
            let queue = self.wgpu_context.get_queue();

            let mut baker = AccelerationBaker::new(
                device,
                queue,
                &create_iblbake_command.file_path,
                create_iblbake_command.bake_info,
            );
            baker.bake(device, queue, &self.shader_library);
            let merge_cube_map = |x: &CubeMap<image::Rgba<f32>, Vec<f32>>| {
                let size = x.negative_x.width();
                let mut merge_image = image::Rgba32FImage::new(size, size * 6);
                let negative_x = x.negative_x.view(0, 0, size, size);
                let positive_x = x.positive_x.view(0, 0, size, size);
                let negative_y = x.negative_y.view(0, 0, size, size);
                let positive_y = x.positive_y.view(0, 0, size, size);
                let negative_z = x.negative_z.view(0, 0, size, size);
                let positive_z = x.positive_z.view(0, 0, size, size);
                for (index, image) in [
                    negative_x, positive_x, negative_y, positive_y, negative_z, positive_z,
                ]
                .iter()
                .enumerate()
                {
                    merge_image
                        .copy_from(image.deref(), 0, size * index as u32)
                        .map_err(|err| crate::error::Error::ImageError(err))?;
                }
                crate::error::Result::Ok(merge_image)
            };
            let save_face = |data: Vec<f32>, mipmaps: u32, save_dir: &Path, name: &str| {
                let surface = image_dds::SurfaceRgba32Float {
                    width: baker.get_bake_info().pre_filter_cube_map_length,
                    height: baker.get_bake_info().pre_filter_cube_map_length,
                    depth: 1,
                    layers: 1,
                    mipmaps,
                    data,
                }
                .encode(
                    image_dds::ImageFormat::BC6hRgbUfloat,
                    image_dds::Quality::Slow,
                    image_dds::Mipmaps::FromSurface,
                )
                .map_err(|err| crate::error::Error::ImageDdsSurface(err))?;
                let dds = surface
                    .to_dds()
                    .map_err(|err| crate::error::Error::ImageDdsCreateDds(err))?;
                let path = save_dir.join(format!("pre_filter_{}.dds", name));
                let file = std::fs::File::create(&path)
                    .map_err(|err| crate::error::Error::IO(err, None))?;
                let mut writer = std::io::BufWriter::new(file);
                dds.write(&mut writer)
                    .map_err(|err| crate::error::Error::DdsFile(err))?;
                crate::error::Result::Ok(())
            };
            let result = (|| {
                let save_dir = create_iblbake_command
                    .save_dir
                    .ok_or(crate::error::Error::Other(None))?;
                if !save_dir.exists() {
                    return Err(crate::error::Error::Other(None));
                }
                let brdflut_image =
                    ibl_readback::IBLReadBack::read_brdflut_texture(&baker, device, queue)?;
                let image = brdflut_image
                    .as_rgba32f()
                    .ok_or(crate::error::Error::Other(None))?;

                let surface = image_dds::SurfaceRgba32Float::from_image_layers(&image, 1)
                    .encode(
                        image_dds::ImageFormat::BC6hRgbUfloat,
                        image_dds::Quality::Slow,
                        image_dds::Mipmaps::Disabled,
                    )
                    .map_err(|err| crate::error::Error::ImageDdsSurface(err))?;

                let dds = surface
                    .to_dds()
                    .map_err(|err| crate::error::Error::ImageDdsCreateDds(err))?;
                let path = save_dir.join("brdf.dds");
                let file = std::fs::File::create(&path)
                    .map_err(|err| crate::error::Error::IO(err, None))?;
                let mut writer = std::io::BufWriter::new(file);
                dds.write(&mut writer)
                    .map_err(|err| crate::error::Error::DdsFile(err))?;

                let irradiance_image = ibl_readback::IBLReadBack::read_irradiance_cube_map_texture(
                    &baker, device, queue,
                )?;
                let dds_image = merge_cube_map(&irradiance_image)?;

                let surface = image_dds::SurfaceRgba32Float::from_image_layers(&dds_image, 6)
                    .encode(
                        image_dds::ImageFormat::BC6hRgbUfloat,
                        image_dds::Quality::Slow,
                        image_dds::Mipmaps::Disabled,
                    )
                    .map_err(|err| crate::error::Error::ImageDdsSurface(err))?;
                let dds = surface
                    .to_dds()
                    .map_err(|err| crate::error::Error::ImageDdsCreateDds(err))?;
                let path = save_dir.join("irradiance.dds");
                let file = std::fs::File::create(&path)
                    .map_err(|err| crate::error::Error::IO(err, None))?;
                let mut writer = std::io::BufWriter::new(file);
                dds.write(&mut writer)
                    .map_err(|err| crate::error::Error::DdsFile(err))?;

                let pre_filter_images =
                    ibl_readback::IBLReadBack::read_pre_filter_cube_map_textures(
                        &baker, device, queue,
                    )?;

                macro_rules! save_face {
                    ($face:ident) => {
                        save_face(
                            pre_filter_images.iter().fold(vec![], |mut acc, x| {
                                acc.extend_from_slice(&x.$face.as_raw());
                                acc
                            }),
                            pre_filter_images.len() as u32,
                            &save_dir,
                            stringify!($face),
                        )?
                    };
                }
                save_face!(negative_x);
                save_face!(negative_y);
                save_face!(negative_z);
                save_face!(positive_x);
                save_face!(positive_y);
                save_face!(positive_z);
                Ok(())
            })();
            match result {
                Ok(_) => {}
                Err(err) => log::warn!("{}", err),
            }
            render_output
                .create_ibl_handles
                .insert(create_iblbake_command.handle);
            self.ibl_bakes.insert(create_iblbake_command.handle, baker);
        }

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
        self.vt_pass();
        self.draw_objects(&output_view);

        self.draw_object_commands.clear();

        while let Some(output) = self.ui_output_commands.pop_front() {
            let device = self.wgpu_context.get_device();
            let queue = self.wgpu_context.get_queue();
            self.gui_renderer
                .render(output, queue, device, &self.screen_descriptor, &output_view)
        }
        texture.present();
        #[cfg(feature = "renderdoc")]
        {
            if is_capture_frame {
                let device = self.wgpu_context.get_device();
                if let Some(render_doc_context) = &mut self.render_doc_context {
                    render_doc_context.stop_capture(device);
                }
            }
        }
        return Some(render_output);
    }

    fn clear_buffer(&self, surface_texture_view: &TextureView, depth_texture_view: &TextureView) {
        self.attachment_pipeline.draw(
            self.wgpu_context.get_device(),
            self.wgpu_context.get_queue(),
            surface_texture_view,
            depth_texture_view,
        );

        if let Some(pass) = &self.virtual_texture_pass {
            pass.begin_new_frame(
                self.wgpu_context.get_device(),
                self.wgpu_context.get_queue(),
            );
        }
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
            RenderCommand::CreateIBLBake(command) => {
                self.create_iblbake_commands.push_back(command)
            }
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
            RenderCommand::Task(command) => self.task_commands.push_back(command),
            #[cfg(feature = "renderdoc")]
            RenderCommand::CaptureFrame => {
                if let Some(render_doc_context) = &mut self.render_doc_context {
                    render_doc_context.capture_commands.push_back(());
                }
            }
            RenderCommand::Settings(settings) => {
                self.set_settings(settings);
            }
            RenderCommand::CreateVirtualTextureSource(command) => {
                if let Some(virtual_texture_pass) = &mut self.virtual_texture_pass {
                    virtual_texture_pass
                        .virtual_texture_sources
                        .insert(command.handle, VirtualTextureSource::new(command.source));
                }
            }
        }
        return None;
    }

    fn vt_pass(&mut self) {
        let Some(virtual_texture_pass) = &mut self.virtual_texture_pass else {
            return;
        };

        let device = self.wgpu_context.get_device();
        let queue = self.wgpu_context.get_queue();
        for draw_object_command in &self.draw_object_commands {
            match &draw_object_command.material_type {
                EMaterialType::Phong(material) => {
                    let mut vertex_buffers = Vec::<&Buffer>::new();
                    let mut index_buffer: Option<&wgpu::Buffer> = None;

                    for vertex_buffer in &draw_object_command.vertex_buffers {
                        if let Some(vertex_buffer) = self.buffers.get(&vertex_buffer) {
                            vertex_buffers.push(vertex_buffer);
                        }
                    }
                    if vertex_buffers.is_empty() {
                        continue;
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
                    virtual_texture_pass.render(
                        device,
                        queue,
                        &[mesh_buffer.clone()],
                        material.constants.model.clone(),
                        material.constants.view.clone(),
                        material.constants.projection.clone(),
                        draw_object_command.id,
                    );
                }
                EMaterialType::PBR(_) => todo!(),
            }
        }
        let result = virtual_texture_pass.parse_feed_back(
            self.wgpu_context.get_device(),
            self.wgpu_context.get_queue(),
        );
        let Ok(result) = result else {
            return;
        };

        let indirect_map = virtual_texture_pass.upload_physical_texture(
            self.wgpu_context.get_device(),
            self.wgpu_context.get_queue(),
            &result,
        );
        virtual_texture_pass.update_indirec_table(self.wgpu_context.get_queue(), indirect_map);
    }

    fn draw_objects(&mut self, surface_texture_view: &wgpu::TextureView) {
        let device = self.wgpu_context.get_device();
        let queue = self.wgpu_context.get_queue();
        for draw_object_command in &self.draw_object_commands {
            match &draw_object_command.material_type {
                EMaterialType::Phong(material) => {
                    let diffuse_texture: &Texture;
                    let specular_texture: &Texture;
                    let binding = self.default_textures.get_white_texture();
                    if let Some(diffuse_texture_type) = &material.diffuse_texture {
                        match diffuse_texture_type {
                            ETextureType::Base(diffuse_texture_handle) => {
                                diffuse_texture = self
                                    .textures
                                    .get(&diffuse_texture_handle)
                                    .unwrap_or(&binding);
                            }
                            ETextureType::Virtual(_) => {
                                diffuse_texture = &binding;
                            }
                        }
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

                    let mut constants = shading::Constants {
                        model: material.constants.model,
                        view: material.constants.view,
                        projection: material.constants.projection,
                        is_enable_virtual_texture: 0,
                        physical_texture_size: glam::Vec2::splat(
                            self.settings.virtual_texture_setting.physical_texture_size as f32,
                        ),
                        tile_size: self.settings.virtual_texture_setting.tile_size as f32,
                        diffuse_texture_size: Default::default(),
                        diffuse_texture_max_lod: Default::default(),
                        is_virtual_diffuse_texture: Default::default(),
                        specular_texture_size: Default::default(),
                        specular_texture_max_lod: Default::default(),
                        is_virtual_specular_texture: Default::default(),
                    };

                    if let Some(virtual_texture_pass) = &self.virtual_texture_pass {
                        if let Some(ETextureType::Virtual(diffuse_texture_handle)) =
                            &material.diffuse_texture
                        {
                            let source = virtual_texture_pass
                                .virtual_texture_sources
                                .get(diffuse_texture_handle);
                            if let Some(source) = source {
                                let max_mips = rs_core_minimal::misc::calculate_max_mips(
                                    source.get_size().min_element(),
                                );
                                let max_lod = max_mips
                                    - self.settings.virtual_texture_setting.tile_size.ilog2()
                                    - 1;
                                constants.diffuse_texture_max_lod = max_lod;
                                constants.diffuse_texture_size = source.get_size().as_vec2();
                                constants.is_virtual_diffuse_texture = 1;
                            }
                        }
                        constants.is_enable_virtual_texture = 1;
                    }

                    let physical_texture_view = match &self.virtual_texture_pass {
                        Some(pass) => pass.get_physical_texture_view(),
                        None => self.default_textures.get_black_texture_view(),
                    };
                    let indirect_table_view = match &self.virtual_texture_pass {
                        Some(pass) => pass.get_indirect_table_view(),
                        None => self.default_textures.get_black_u32_texture_view(),
                    };
                    self.shading_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &self.depth_texture.get_view(),
                        &constants,
                        &[mesh_buffer],
                        &diffuse_texture_view,
                        &specular_texture_view,
                        &physical_texture_view,
                        &indirect_table_view,
                    );
                }
                EMaterialType::PBR(_) => todo!(),
            }
        }
    }

    fn set_settings(&mut self, settings: RenderSettings) {
        if settings.virtual_texture_setting.is_enable {
            if let Some(_) = &mut self.virtual_texture_pass {
            } else {
                let surface_width = self.wgpu_context.get_surface_config().width;
                let surface_height = self.wgpu_context.get_surface_config().height;
                self.virtual_texture_pass = VirtualTexturePass::new(
                    self.wgpu_context.get_device(),
                    &self.shader_library,
                    false,
                    glam::uvec2(surface_width, surface_height),
                    settings.virtual_texture_setting.clone(),
                )
                .ok();
            }
        } else {
            self.virtual_texture_pass = None;
        }
        self.settings = settings;
    }

    fn surface_size_will_change(&mut self, new_size: glam::UVec2) {
        let width = new_size.x;
        let height = new_size.y;
        self.screen_descriptor.size_in_pixels[0] = width;
        self.screen_descriptor.size_in_pixels[1] = height;
        self.wgpu_context.window_resized(width, height);
        let device = self.wgpu_context.get_device();
        self.depth_texture = DepthTexture::new(width, height, device, Some("Base.DepthTexture"));

        if let Some(virtual_texture_pass) = &mut self.virtual_texture_pass {
            virtual_texture_pass.change_surface_size(device, new_size);
        }
    }
}
