use crate::acceleration_bake::AccelerationBaker;
use crate::base_render_pipeline_pool::BaseRenderPipelinePool;
use crate::cube_map::CubeMap;
use crate::default_textures::DefaultTextures;
use crate::depth_texture::DepthTexture;
use crate::error::Result;
use crate::gpu_vertex_buffer::GpuVertexBufferImp;
use crate::prebake_ibl::PrebakeIBL;
use crate::render_pipeline::attachment_pipeline::AttachmentPipeline;
use crate::render_pipeline::grid_pipeline::GridPipeline;
use crate::render_pipeline::material_pipeline::MaterialRenderPipeline;
use crate::render_pipeline::shading::ShadingPipeline;
use crate::render_pipeline::skin_mesh_shading::SkinMeshShadingPipeline;
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

pub const SKIN_MESH_RENDER_PIPELINE: &str = "SKIN_MESH_RENDER_PIPELINE";
pub const STATIC_MESH_RENDER_PIPELINE: &str = "STATIC_MESH_RENDER_PIPELINE";
pub const GRID_RENDER_PIPELINE: &str = "GRID_RENDER_PIPELINE";

pub struct Renderer {
    wgpu_context: WGPUContext,
    gui_renderer: EGUIRenderer,
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
    samplers: HashMap<u64, Sampler>,

    shading_pipeline: ShadingPipeline,
    skin_mesh_shading_pipeline: SkinMeshShadingPipeline,
    grid_render_pipeline: GridPipeline,
    attachment_pipeline: AttachmentPipeline,

    depth_textures: HashMap<isize, DepthTexture>,
    default_textures: DefaultTextures,

    texture_descriptors: HashMap<u64, TextureDescriptorCreateInfo>,
    buffer_infos: HashMap<u64, BufferCreateInfo>,

    #[cfg(feature = "renderdoc")]
    render_doc_context: Option<crate::renderdoc::Context>,

    virtual_texture_pass: Option<VirtualTexturePass>,

    settings: RenderSettings,

    base_render_pipeline_pool: BaseRenderPipelinePool,

    main_window_id: isize,

    material_render_pipelines: HashMap<MaterialRenderPipelineHandle, MaterialRenderPipeline>,

    prebake_ibls: HashMap<u64, PrebakeIBL>,
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
        let main_window_id = {
            let binding = wgpu_context.get_window_ids();
            *binding.first().expect("Not null")
        };
        let current_swapchain_format = wgpu_context.get_current_swapchain_format(main_window_id);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [surface_width, surface_height],
            pixels_per_point: scale_factor,
        };
        let egui_render_pass = EGUIRenderer::new(
            wgpu_context.get_device(),
            current_swapchain_format,
            1,
            HashMap::from([(main_window_id, screen_descriptor)]),
        );

        let mut shader_library = ShaderLibrary::new();
        shader_library.load_shaders_from(shaders, wgpu_context.get_device());
        let mut sampler_cache = SamplerCache::new();
        let mut base_render_pipeline_pool = BaseRenderPipelinePool::default();
        let shading_pipeline = ShadingPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            false,
        );
        let skin_mesh_shading_pipeline = SkinMeshShadingPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            &mut base_render_pipeline_pool,
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
            &current_swapchain_format,
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
        let grid_render_pipeline = GridPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            &mut base_render_pipeline_pool,
        );

        Renderer {
            wgpu_context,
            gui_renderer: egui_render_pass,
            // screen_descriptor,
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
            depth_textures: HashMap::from([(main_window_id, depth_texture)]),
            default_textures,
            texture_descriptors: HashMap::new(),
            buffer_infos: HashMap::new(),
            task_commands: VecDeque::new(),
            ibl_bakes: HashMap::new(),
            #[cfg(feature = "renderdoc")]
            render_doc_context: crate::renderdoc::Context::new().ok(),
            virtual_texture_pass,
            settings,
            skin_mesh_shading_pipeline,
            base_render_pipeline_pool,
            samplers: HashMap::new(),
            grid_render_pipeline,
            main_window_id,
            material_render_pipelines: HashMap::new(),
            prebake_ibls: HashMap::new(),
        }
    }

    pub fn from_window<W>(
        window_id: isize,
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
            window_id,
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
                backends: match settings.get_backends_platform() {
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
        window_id: isize,
        window: &W,
        surface_width: u32,
        surface_height: u32,
    ) -> Result<()>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let device = self.wgpu_context.get_device();
        let depth_texture = DepthTexture::new(
            surface_width,
            surface_height,
            device,
            Some(&format!("Base.DepthTexture.{}", window_id)),
        );
        self.depth_textures.insert(window_id, depth_texture);
        let scale_factor = 1.0;
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [surface_width, surface_height],
            pixels_per_point: scale_factor,
        };
        self.gui_renderer
            .add_screen_descriptor(window_id, screen_descriptor);
        self.wgpu_context
            .set_new_window(window_id, window, surface_width, surface_height)
    }

    pub fn present(&mut self, window_id: isize) -> Option<RenderOutput> {
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
            self.surface_size_will_change(
                resize_command.window_id,
                glam::uvec2(resize_command.width, resize_command.height),
            );
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
            let save_data_as_dds = |data: &[f32],
                                    width: u32,
                                    height: u32,
                                    layers: u32,
                                    mipmaps: u32,
                                    save_dir: &Path,
                                    name: &str| {
                let surface = image_dds::SurfaceRgba32Float {
                    width,
                    height,
                    depth: 1,
                    layers,
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
                let path = save_dir.join(format!("{}.dds", name));
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
                let brdflut_image = brdflut_image
                    .as_rgba32f()
                    .ok_or(crate::error::Error::Other(None))?;
                save_data_as_dds(
                    brdflut_image.as_ref(),
                    brdflut_image.width(),
                    brdflut_image.height(),
                    1,
                    1,
                    &save_dir,
                    "brdf",
                )?;

                let irradiance_image = ibl_readback::IBLReadBack::read_irradiance_cube_map_texture(
                    &baker, device, queue,
                )?;
                let irradiance_image = merge_cube_map(&irradiance_image)?;
                save_data_as_dds(
                    irradiance_image.as_ref(),
                    irradiance_image.width(),
                    irradiance_image.height() / 6,
                    6,
                    1,
                    &save_dir,
                    "irradiance",
                )?;

                let pre_filter_images =
                    ibl_readback::IBLReadBack::read_pre_filter_cube_map_textures(
                        &baker, device, queue,
                    )?;
                let mut data: Vec<f32> = vec![];
                macro_rules! merge_face_layer_data {
                    ($face:ident) => {
                        let mut layer_data: Vec<f32> = vec![];
                        for cube_map_mipmap in pre_filter_images.iter() {
                            let data = cube_map_mipmap.$face.as_ref();
                            layer_data.extend_from_slice(data);
                        }
                        data.extend(layer_data);
                    };
                }
                merge_face_layer_data!(negative_x);
                merge_face_layer_data!(negative_y);
                merge_face_layer_data!(negative_z);
                merge_face_layer_data!(positive_x);
                merge_face_layer_data!(positive_y);
                merge_face_layer_data!(positive_z);
                save_data_as_dds(
                    data.as_ref(),
                    baker.get_bake_info().pre_filter_cube_map_length,
                    baker.get_bake_info().pre_filter_cube_map_length,
                    6,
                    pre_filter_images.len() as u32,
                    &save_dir,
                    "pre_filter",
                )?;
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

        let texture = match self.wgpu_context.get_current_surface_texture(window_id) {
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
        let depth_texture_view = &self
            .depth_textures
            .get(&window_id)
            .expect("Not null")
            .get_view();
        self.clear_buffer(&output_view, depth_texture_view);
        self.vt_pass();
        self.draw_objects(&output_view);

        self.draw_object_commands.clear();

        for output in self
            .ui_output_commands
            .iter()
            .filter(|x| x.window_id == window_id)
        {
            let device = self.wgpu_context.get_device();
            let queue = self.wgpu_context.get_queue();
            self.gui_renderer
                .render(output, queue, device, &output_view);
        }
        self.ui_output_commands.retain(|x| x.window_id != window_id);

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
            Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
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
        K: AsRef<str>,
    {
        self.shader_library
            .load_shaders_from(shaders, self.wgpu_context.get_device());
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
            RenderCommand::Present(window_id) => {
                return self.present(window_id);
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
            RenderCommand::ChangeViewMode(new_view_mode) => {
                self.skin_mesh_shading_pipeline.set_view_mode(
                    new_view_mode,
                    self.wgpu_context.get_device(),
                    &self.shader_library,
                    &mut self.base_render_pipeline_pool,
                );
            }
            RenderCommand::CreateSampler(create_sampler) => {
                let sampler = self
                    .wgpu_context
                    .get_device()
                    .create_sampler(&create_sampler.sampler_descriptor);
                self.samplers.insert(create_sampler.handle, sampler);
            }
            RenderCommand::RemoveWindow(window_id) => {
                self.wgpu_context.remove_window(window_id);
                self.depth_textures.remove(&window_id);
                self.gui_renderer.remove_screen_descriptor(window_id);
            }
            RenderCommand::CreateMaterialRenderPipeline(create_render_pipeline) => {
                let device = self.wgpu_context.get_device();
                let name = ShaderLibrary::get_material_shader_name(create_render_pipeline.handle);
                let create_shader_result = self.shader_library.load_shader_from(
                    name.clone(),
                    create_render_pipeline.shader_code,
                    device,
                );
                match create_shader_result {
                    Ok(_) => {}
                    Err(err) => match err {
                        crate::error::Error::ShaderReflection(_, _) => {}
                        crate::error::Error::Wgpu(err) => match err {
                            Error::OutOfMemory { .. } => {
                                todo!()
                            }
                            Error::Validation { description, .. } => {
                                log::trace!("Failed to create shader, {}", description);
                            }
                        },
                        _ => unreachable!(),
                    },
                }
                let current_swapchain_format = self
                    .wgpu_context
                    .get_current_swapchain_format(self.main_window_id);

                let material_render_pipeline = MaterialRenderPipeline::new(
                    create_render_pipeline.handle,
                    device,
                    &self.shader_library,
                    &current_swapchain_format,
                );
                if let Ok(material_render_pipeline) = material_render_pipeline {
                    self.material_render_pipelines
                        .insert(create_render_pipeline.handle, material_render_pipeline);
                    log::trace!("Create material render pipeline: {}", name);
                }
            }
            RenderCommand::UploadPrebakeIBL(upload_prebake_ibl) => {
                macro_rules! get_surface {
                    ($name:ident, $name1:ident) => {
                        let $name = (|| {
                            let reader = std::io::Cursor::new(&upload_prebake_ibl.$name1);
                            let dds = ddsfile::Dds::read(reader)
                                .map_err(|err| crate::error::Error::DdsFile(err))?;
                            let surface = image_dds::Surface::from_dds(&dds)
                                .map_err(|err| crate::error::Error::ImageDdsSurface(err))?;
                            surface
                                .decode_rgbaf32()
                                .map_err(|err| crate::error::Error::ImageDdsSurface(err))
                        })();
                    };
                }
                get_surface!(brdf_surface, brdf_data);
                get_surface!(irradiance_surface, irradiance_data);
                get_surface!(pre_filter_surface, pre_filter_data);
                let prebake_ibl = (|| {
                    let device = self.wgpu_context.get_device();
                    let queue = self.wgpu_context.get_queue();
                    let brdf_surface = brdf_surface?;
                    let irradiance_surface = irradiance_surface?;
                    let pre_filter_surface = pre_filter_surface?;
                    let prebake = PrebakeIBL::from_surfaces(
                        device,
                        queue,
                        brdf_surface,
                        irradiance_surface,
                        pre_filter_surface,
                    );
                    prebake
                })();
                match prebake_ibl {
                    Ok(prebake_ibl) => {
                        self.prebake_ibls
                            .insert(upload_prebake_ibl.handle, prebake_ibl);
                    }
                    Err(err) => {
                        log::trace!("{}", err);
                    }
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
            let is_virtual = draw_object_command
                .binding_resources
                .iter()
                .flatten()
                .find(|x| match x {
                    EBindingResource::Texture(texture) => match texture {
                        ETextureType::Virtual(_) => true,
                        _ => false,
                    },
                    _ => false,
                })
                .is_some();
            if !is_virtual {
                continue;
            }
            let mut vertex_buffers = Vec::<&Buffer>::new();
            let mut index_buffer: Option<&wgpu::Buffer> = None;

            match draw_object_command.render_pipeline.as_str() {
                SKIN_MESH_RENDER_PIPELINE => {
                    vertex_buffers.push(
                        self.buffers
                            .get(&draw_object_command.vertex_buffers[0])
                            .unwrap(),
                    );
                    vertex_buffers.push(
                        self.buffers
                            .get(&draw_object_command.vertex_buffers[2])
                            .unwrap(),
                    );
                }
                STATIC_MESH_RENDER_PIPELINE => {
                    vertex_buffers.push(
                        self.buffers
                            .get(&draw_object_command.vertex_buffers[0])
                            .unwrap(),
                    );
                }
                _ => {}
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

            let mut group_binding_resource: Vec<Vec<BindingResource>> = vec![vec![], vec![]];

            for binding_resource_type in draw_object_command.global_binding_resources.iter() {
                let binding_resource = group_binding_resource.get_mut(0).unwrap();
                match binding_resource_type {
                    EBindingResource::Texture(_) => {}
                    EBindingResource::Constants(buffer_handle) => {
                        let buffer = self.buffers.get(buffer_handle).unwrap().as_entire_binding();
                        binding_resource.push(buffer);
                    }
                    EBindingResource::Sampler(_) => {}
                }
            }

            for binding_resource_type in draw_object_command.vt_binding_resources.iter() {
                let binding_resource = group_binding_resource.get_mut(1).unwrap();
                match binding_resource_type {
                    EBindingResource::Texture(_) => {}
                    EBindingResource::Constants(buffer_handle) => {
                        let buffer = self.buffers.get(buffer_handle).unwrap().as_entire_binding();
                        binding_resource.push(buffer);
                    }
                    EBindingResource::Sampler(_) => {}
                }
            }

            match draw_object_command.render_pipeline.as_str() {
                SKIN_MESH_RENDER_PIPELINE => {
                    virtual_texture_pass.render(
                        device,
                        queue,
                        &[mesh_buffer.clone()],
                        group_binding_resource,
                        false,
                    );
                }
                STATIC_MESH_RENDER_PIPELINE => {
                    virtual_texture_pass.render(
                        device,
                        queue,
                        &[mesh_buffer.clone()],
                        group_binding_resource,
                        true,
                    );
                }
                _ => {}
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

            enum _ResourceType<'a> {
                Buffer(&'a Buffer),
                TextureView(TextureView),
                Sampler(&'a Sampler),
            }

            let mut group_binding_resource: Vec<Vec<BindingResource>> = vec![vec![]];
            let mut global_resources: Vec<_ResourceType> = Vec::new();

            for global_binding_resource in draw_object_command.global_binding_resources.iter() {
                match global_binding_resource {
                    EBindingResource::Texture(_) => {}
                    EBindingResource::Constants(handle) => {
                        let buffer = self.buffers.get(handle).unwrap();
                        global_resources.push(_ResourceType::Buffer(buffer));
                    }
                    EBindingResource::Sampler(handle) => {
                        let sampler = self.samplers.get(handle).unwrap();
                        global_resources.push(_ResourceType::Sampler(sampler));
                    }
                }
            }

            let physical_texture_view = match &self.virtual_texture_pass {
                Some(pass) => pass.get_physical_texture_view(),
                None => self.default_textures.get_black_texture_view(),
            };
            let indirect_table_view = match &self.virtual_texture_pass {
                Some(pass) => pass.get_indirect_table_view(),
                None => self.default_textures.get_black_u32_texture_view(),
            };
            global_resources.push(_ResourceType::TextureView(physical_texture_view));
            global_resources.push(_ResourceType::TextureView(indirect_table_view));
            for resource in global_resources.iter() {
                match resource {
                    _ResourceType::Buffer(buffer) => {
                        group_binding_resource[0].push(buffer.as_entire_binding());
                    }
                    _ResourceType::TextureView(texture_view) => {
                        group_binding_resource[0].push(BindingResource::TextureView(texture_view));
                    }
                    _ResourceType::Sampler(sampler) => {
                        group_binding_resource[0].push(BindingResource::Sampler(sampler));
                    }
                }
            }

            let mut resources: Vec<Vec<_ResourceType>> =
                Vec::with_capacity(draw_object_command.binding_resources.len());

            for binding_resources in draw_object_command.binding_resources.iter() {
                let mut binding_resource: Vec<_ResourceType> =
                    Vec::with_capacity(binding_resources.len());
                for binding_resource_type in binding_resources.iter() {
                    match binding_resource_type {
                        EBindingResource::Texture(texture_type) => match texture_type {
                            ETextureType::Base(texture_handle) => {
                                let binding = self.default_textures.get_white_texture();
                                let texture =
                                    self.textures.get(&texture_handle).unwrap_or(&binding);
                                let texture_view =
                                    texture.create_view(&TextureViewDescriptor::default());
                                binding_resource.push(_ResourceType::TextureView(texture_view));
                            }
                            ETextureType::Virtual(_) => {
                                let binding = self.default_textures.get_white_texture();
                                let texture_view =
                                    binding.create_view(&TextureViewDescriptor::default());
                                binding_resource.push(_ResourceType::TextureView(texture_view));
                            }
                            ETextureType::None => {
                                let binding = self.default_textures.get_white_texture();
                                let texture_view =
                                    binding.create_view(&TextureViewDescriptor::default());
                                binding_resource.push(_ResourceType::TextureView(texture_view));
                            }
                        },
                        EBindingResource::Constants(buffer_handle) => {
                            binding_resource.push(_ResourceType::Buffer(
                                self.buffers.get(buffer_handle).unwrap(),
                            ));
                        }
                        EBindingResource::Sampler(handle) => {
                            let sampler = self.samplers.get(handle).unwrap();
                            binding_resource.push(_ResourceType::Sampler(sampler));
                        }
                    }
                }
                resources.push(binding_resource);
            }

            for resource in resources.iter() {
                let mut binding_resource: Vec<BindingResource> = Vec::with_capacity(resource.len());
                for resource_type in resource.iter() {
                    match resource_type {
                        _ResourceType::Buffer(buffer) => {
                            binding_resource.push(buffer.as_entire_binding());
                        }
                        _ResourceType::TextureView(texture_view) => {
                            binding_resource.push(BindingResource::TextureView(texture_view));
                        }
                        _ResourceType::Sampler(sampler) => {
                            binding_resource.push(BindingResource::Sampler(sampler));
                        }
                    }
                }
                group_binding_resource.push(binding_resource);
            }
            let depth_texture_view = self
                .depth_textures
                .get(&self.main_window_id)
                .unwrap()
                .get_view();
            match draw_object_command.render_pipeline.as_str() {
                SKIN_MESH_RENDER_PIPELINE => {
                    self.skin_mesh_shading_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &depth_texture_view,
                        &[mesh_buffer],
                        group_binding_resource,
                    );
                }
                STATIC_MESH_RENDER_PIPELINE => {
                    self.shading_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &depth_texture_view,
                        &[mesh_buffer],
                        group_binding_resource,
                    );
                }
                GRID_RENDER_PIPELINE => {
                    self.grid_render_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &depth_texture_view,
                        &[mesh_buffer],
                        vec![vec![group_binding_resource[0][0].clone()]],
                    );
                }
                _ => {
                    let any_first = self.ibl_bakes.iter().find(|_| true);
                    let any_first1 = self.prebake_ibls.iter().find(|_| true);
                    let brdflut_texture_view = if let Some((_, acceleration_baker)) = any_first {
                        Some(acceleration_baker.get_brdflut_texture_view())
                    } else if let Some((_, prebake_ibl)) = any_first1 {
                        Some(prebake_ibl.get_brdflut_texture_view())
                    } else {
                        None
                    };
                    let pre_filter_texture_view = if let Some((_, acceleration_baker)) = any_first {
                        Some(acceleration_baker.get_pre_filter_cube_map_texture_view())
                    } else if let Some((_, prebake_ibl)) = any_first1 {
                        Some(prebake_ibl.get_pre_filter_cube_map_texture_view())
                    } else {
                        None
                    };
                    let irradiance_texture_view = if let Some((_, acceleration_baker)) = any_first {
                        Some(acceleration_baker.get_irradiance_texture_view())
                    } else if let Some((_, prebake_ibl)) = any_first1 {
                        Some(prebake_ibl.get_irradiance_texture_view())
                    } else {
                        None
                    };
                    if let (
                        Some(brdflut_texture_view),
                        Some(pre_filter_texture_view),
                        Some(irradiance_texture_view),
                    ) = (
                        brdflut_texture_view,
                        pre_filter_texture_view,
                        irradiance_texture_view,
                    ) {
                        let brdf_binding_resource =
                            BindingResource::TextureView(&brdflut_texture_view);
                        let pre_filter_binding_resource =
                            BindingResource::TextureView(&pre_filter_texture_view);
                        let irradiance_binding_resource =
                            BindingResource::TextureView(&irradiance_texture_view);

                        group_binding_resource[0].push(brdf_binding_resource);
                        group_binding_resource[0].push(pre_filter_binding_resource);
                        group_binding_resource[0].push(irradiance_binding_resource);

                        (|| {
                            if !draw_object_command.render_pipeline.starts_with("material_") {
                                return;
                            }
                            let handle = draw_object_command
                                .render_pipeline
                                .strip_prefix("material_")
                                .unwrap()
                                .parse::<MaterialRenderPipelineHandle>();
                            let Ok(handle) = handle else {
                                return;
                            };
                            let material_render_pipeline =
                                self.material_render_pipelines.get(&handle);
                            let Some(material_render_pipeline) = material_render_pipeline else {
                                return;
                            };
                            material_render_pipeline.draw(
                                device,
                                queue,
                                surface_texture_view,
                                &depth_texture_view,
                                &[mesh_buffer],
                                group_binding_resource,
                            );
                        })();
                    }
                }
            }
        }
    }

    fn set_settings(&mut self, settings: RenderSettings) {
        if settings.virtual_texture_setting.is_enable {
            if let Some(_) = &mut self.virtual_texture_pass {
            } else {
                let surface_width = self
                    .wgpu_context
                    .get_surface_config(self.main_window_id)
                    .width;
                let surface_height = self
                    .wgpu_context
                    .get_surface_config(self.main_window_id)
                    .height;
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

    fn surface_size_will_change(&mut self, window_id: isize, new_size: glam::UVec2) {
        let width = new_size.x;
        let height = new_size.y;
        self.gui_renderer.change_size(window_id, width, height);
        self.wgpu_context.window_resized(window_id, width, height);
        let device = self.wgpu_context.get_device();
        let depth_texture = DepthTexture::new(
            width,
            height,
            device,
            Some(&format!("Base.DepthTexture.{}", window_id)),
        );
        self.depth_textures.insert(window_id, depth_texture);
        if let Some(virtual_texture_pass) = &mut self.virtual_texture_pass {
            virtual_texture_pass.change_surface_size(device, new_size);
        }
    }
}
