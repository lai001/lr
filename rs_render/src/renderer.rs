use crate::acceleration_bake::AccelerationBaker;
use crate::antialias_type::EAntialiasType;
use crate::base_compute_pipeline_pool::BaseComputePipelinePool;
use crate::base_render_pipeline_pool::BaseRenderPipelinePool;
use crate::compute_pipeline::light_culling::LightCullingComputePipeline;
use crate::cube_map::CubeMap;
use crate::depth_texture::DepthTexture;
use crate::error::Result;
use crate::gpu_vertex_buffer::GpuVertexBufferImp;
use crate::light_culling::LightCulling;
use crate::prebake_ibl::PrebakeIBL;
use crate::render_pipeline::attachment_pipeline::{AttachmentPipeline, ClearDepth, EClearType};
use crate::render_pipeline::fxaa::FXAAPipeline;
use crate::render_pipeline::grid_pipeline::GridPipeline;
use crate::render_pipeline::material_pipeline::VariantMaterialRenderPipeline;
use crate::render_pipeline::mesh_view::MeshViewPipeline;
use crate::render_pipeline::mesh_view_multiple_draw::MeshViewMultipleDrawPipeline;
use crate::render_pipeline::particle_pipeline::ParticlePipeline;
use crate::render_pipeline::primitive::PrimitiveRenderPipeline;
use crate::render_pipeline::shading::ShadingPipeline;
use crate::render_pipeline::skin_mesh_shading::SkinMeshShadingPipeline;
use crate::shader_library::ShaderLibrary;
use crate::shadow_pass::ShadowPipilines;
use crate::virtual_texture_pass::VirtualTexturePass;
use crate::virtual_texture_source::VirtualTextureSource;
use crate::{command::*, ibl_readback, shadow_pass};
use crate::{egui_render::EGUIRenderer, wgpu_context::WGPUContext};
use image::{GenericImage, GenericImageView};
use rs_core_minimal::settings::{self, RenderSettings};
use rs_core_minimal::thread_pool::ThreadPool;
use rs_render_types::MaterialOptions;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

#[derive(Clone)]
pub enum EBuiltinPipelineType {
    SkinMeshPhong,
    StaticMeshPhong,
    Grid,
    MeshView,
    MeshViewMultipleDraw,
    ShadowDepthSkinMesh,
    ShadowDepthStaticMesh,
    Particle,
    Primitive(Option<wgpu::PrimitiveState>),
}

#[derive(Clone)]
pub struct MaterialPipelineType {
    pub handle: MaterialRenderPipelineHandle,
    pub options: MaterialOptions,
}

#[derive(Clone)]
pub enum EPipelineType {
    Builtin(EBuiltinPipelineType),
    Material(MaterialPipelineType),
}

pub struct Renderer {
    wgpu_context: WGPUContext,
    gui_renderer: EGUIRenderer,
    shader_library: ShaderLibrary,

    textures: HashMap<u64, Texture>,
    texture_views: HashMap<u64, TextureView>,
    buffers: HashMap<u64, Arc<Buffer>>,
    ui_textures: HashMap<u64, egui::TextureId>,
    ibl_bakes: HashMap<IBLTexturesKey, AccelerationBaker>,
    samplers: HashMap<u64, Sampler>,

    shading_pipeline: ShadingPipeline,
    skin_mesh_shading_pipeline: SkinMeshShadingPipeline,
    grid_render_pipeline: GridPipeline,
    attachment_pipeline: AttachmentPipeline,
    mesh_view_pipeline: MeshViewPipeline,
    mesh_view_multiple_draw_pipeline: MeshViewMultipleDrawPipeline,
    particle_pipeline: ParticlePipeline,
    primitive_render_pipeline: PrimitiveRenderPipeline,
    light_culling_compute_pipeline: LightCullingComputePipeline,

    depth_textures: HashMap<isize, DepthTexture>,
    // default_textures: DefaultTextures,
    texture_descriptors: HashMap<u64, TextureDescriptorCreateInfo>,
    buffer_infos: HashMap<u64, BufferCreateInfo>,

    #[cfg(feature = "renderdoc")]
    render_doc_context: Option<crate::renderdoc::Context>,

    // virtual_texture_pass: Option<VirtualTexturePass>,
    virtual_texture_pass: HashMap<VirtualTexturePassKey, VirtualTexturePass>,

    settings: RenderSettings,

    base_render_pipeline_pool: BaseRenderPipelinePool,
    base_compute_pipeline_pool: BaseComputePipelinePool,

    main_window_id: isize,

    material_render_pipelines: HashMap<MaterialRenderPipelineHandle, VariantMaterialRenderPipeline>,

    prebake_ibls: HashMap<IBLTexturesKey, PrebakeIBL>,

    shadow_pipilines: Option<shadow_pass::ShadowPipilines>,

    fxaa_pipeline: Option<FXAAPipeline>,

    is_enable_multiple_thread: bool,

    surface_textures: HashMap<isize, SurfaceTexture>,

    bind_groups_collection: moka::sync::Cache<u64, Arc<Vec<BindGroup>>>,

    light_culling: crate::light_culling::LightCulling,
}

impl Renderer {
    pub fn from_context(
        wgpu_context: WGPUContext,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
        shaders: HashMap<String, String>,
        settings: RenderSettings,
    ) -> Result<Renderer> {
        let _span = tracy_client::span!();
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
        let load_shader_results =
            shader_library.load_shaders_from(shaders, wgpu_context.get_device());
        for (shader_name, result) in load_shader_results {
            match result {
                Ok(_) => {}
                Err(err) => match err {
                    crate::error::Error::Wgpu(err) => match err.lock().unwrap().deref() {
                        Error::Validation { description, .. } => {
                            log::warn!("{shader_name}\n{}", description);
                        }
                        _ => {}
                    },
                    _ => {
                        log::warn!("{shader_name}\n{}", err);
                    }
                },
            }
        }
        let mut base_render_pipeline_pool = BaseRenderPipelinePool::default();
        let base_compute_pipeline_pool = BaseComputePipelinePool::default();
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
        let attachment_pipeline = AttachmentPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            &mut base_render_pipeline_pool,
        );

        let grid_render_pipeline = GridPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            &mut base_render_pipeline_pool,
        );

        let mesh_view_pipeline = MeshViewPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            &mut base_render_pipeline_pool,
        );

        let mesh_view_multiple_draw_pipeline = MeshViewMultipleDrawPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            &mut base_render_pipeline_pool,
        );

        let shadow_pipilines = ShadowPipilines::new(
            wgpu_context.get_device(),
            &shader_library,
            &mut base_render_pipeline_pool,
        );

        let fxaa_pipeline = FXAAPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            &mut base_render_pipeline_pool,
        );

        let particle_pipeline = ParticlePipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            &mut base_render_pipeline_pool,
        );

        let primitive_render_pipeline = PrimitiveRenderPipeline::new(
            wgpu_context.get_device(),
            &shader_library,
            &current_swapchain_format,
            &mut base_render_pipeline_pool,
        )?;

        let light_culling_compute_pipeline =
            LightCullingComputePipeline::new(wgpu_context.get_device(), &shader_library)?;

        let is_enable_multiple_thread = settings.is_enable_multithread_rendering;
        let renderer = Renderer {
            wgpu_context,
            gui_renderer: egui_render_pass,
            // screen_descriptor,
            shader_library,

            textures: HashMap::new(),
            buffers: HashMap::new(),
            ui_textures: HashMap::new(),
            shading_pipeline,
            attachment_pipeline,
            depth_textures: HashMap::from([(main_window_id, depth_texture)]),
            texture_descriptors: HashMap::new(),
            buffer_infos: HashMap::new(),

            ibl_bakes: HashMap::new(),
            #[cfg(feature = "renderdoc")]
            render_doc_context: crate::renderdoc::Context::new().ok(),
            virtual_texture_pass: HashMap::new(),
            settings,
            skin_mesh_shading_pipeline,
            base_render_pipeline_pool,
            samplers: HashMap::new(),
            grid_render_pipeline,
            main_window_id,
            material_render_pipelines: HashMap::new(),
            prebake_ibls: HashMap::new(),
            mesh_view_pipeline,
            mesh_view_multiple_draw_pipeline,
            shadow_pipilines: Some(shadow_pipilines),
            base_compute_pipeline_pool,
            fxaa_pipeline: Some(fxaa_pipeline),
            particle_pipeline,
            is_enable_multiple_thread,
            primitive_render_pipeline,
            texture_views: HashMap::new(),
            surface_textures: HashMap::new(),
            bind_groups_collection: moka::sync::Cache::new(1000),
            light_culling_compute_pipeline,
            light_culling: LightCulling::new(),
        };
        Ok(renderer)
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
        let _span = tracy_client::span!();
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
                flags: match settings.is_enable_debugging {
                    true => wgpu::InstanceFlags::debugging(),
                    false => wgpu::InstanceFlags::default(),
                },
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
                backends: match settings.get_backends_platform() {
                    settings::Backends::Primary => Backends::PRIMARY,
                    settings::Backends::Vulkan => Backends::VULKAN,
                    settings::Backends::GL => Backends::GL,
                    settings::Backends::DX12 => Backends::DX12,
                },
            }),
        )?;

        Self::from_context(
            wgpu_context,
            surface_width,
            surface_height,
            scale_factor,
            shaders,
            settings,
        )
    }

    pub fn set_new_window<W>(
        &mut self,
        window_id: isize,
        window: &W,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
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

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [surface_width, surface_height],
            pixels_per_point: scale_factor,
        };
        self.gui_renderer
            .add_screen_descriptor(window_id, screen_descriptor);
        self.wgpu_context
            .set_new_window(window_id, window, surface_width, surface_height)
    }

    pub fn renderdoc_start_capture(&mut self) {
        #[cfg(feature = "renderdoc")]
        {
            if let Some(render_doc_context) = &mut self.render_doc_context {
                render_doc_context.start_capture(self.wgpu_context.get_device());
            }
        }
    }

    pub fn renderdoc_stop_capture(&mut self) {
        #[cfg(feature = "renderdoc")]
        {
            let device = self.wgpu_context.get_device();
            if let Some(render_doc_context) = &mut self.render_doc_context {
                render_doc_context.stop_capture(device);
            }
        }
    }

    fn process_create_iblbake_command(
        &mut self,
        create_iblbake_command: CreateIBLBake,
    ) -> IBLTexturesKey {
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
            let file =
                std::fs::File::create(&path).map_err(|err| crate::error::Error::IO(err, None))?;
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

            let irradiance_image =
                ibl_readback::IBLReadBack::read_irradiance_cube_map_texture(&baker, device, queue)?;
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

            let pre_filter_images = ibl_readback::IBLReadBack::read_pre_filter_cube_map_textures(
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

        self.ibl_bakes
            .insert(create_iblbake_command.key.clone(), baker);
        create_iblbake_command.key
    }

    pub fn present(&mut self, present_info: PresentInfo) {
        let _span = tracy_client::span!();
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

        let msaa_texture_view: Option<TextureView> = match &present_info.scene_viewport.anti_type {
            EAntialiasType::None => None,
            EAntialiasType::FXAA(_) => None,
            EAntialiasType::MSAA(msaa_info) => self
                .textures
                .get(&msaa_info.texture)
                .map(|x| x.create_view(&TextureViewDescriptor::default())),
        };
        let msaa_depth_texture_view: Option<TextureView> =
            match &present_info.scene_viewport.anti_type {
                EAntialiasType::None => None,
                EAntialiasType::FXAA(_) => None,
                EAntialiasType::MSAA(msaa_info) => self
                    .textures
                    .get(&msaa_info.depth_texture)
                    .map(|x| x.create_view(&TextureViewDescriptor::default())),
            };

        // if let (Some(msaa_texture_view), Some(msaa_depth_texture_view)) =
        //     (msaa_texture_view.as_ref(), msaa_depth_texture_view.as_ref())
        // {
        //     self.clear_buffer(
        //         &output_view,
        //         &msaa_depth_texture_view,
        //         Some(msaa_texture_view),
        //     );
        // } else {
        //     let depth_texture = self.depth_textures.get(&window_id).expect("Not null");
        //     let depth_texture_view = depth_texture.get_view();
        //     self.clear_buffer(&output_view, &depth_texture_view, None);
        // }
        self.bind_groups_collection.run_pending_tasks();
        if let Some(scene_light) = &present_info.scene_light {
            self.light_culling_pass(scene_light);
        }
        self.vt_pass(&present_info);
        self.shadow_for_draw_objects(
            present_info.draw_objects.as_slice(),
            present_info.depth_texture_handle,
        );

        let color_texture = match present_info.render_target_type {
            ERenderTargetType::SurfaceTexture(window_id) => {
                match self.surface_textures.get(&window_id) {
                    Some(surface_texture) => {
                        let color_texture = &surface_texture.texture;
                        color_texture
                    }
                    None => return,
                }
            }
            ERenderTargetType::FrameBuffer(options) => {
                let FrameBufferOptions { color, .. } = options;
                match self.textures.get(&color) {
                    Some(texture) => texture,
                    None => return,
                }
            }
        };

        let depth_texture = match present_info.render_target_type {
            ERenderTargetType::SurfaceTexture(window_id) => {
                match self.depth_textures.get(&window_id) {
                    Some(surface_texture) => &surface_texture.depth_texture,
                    None => return,
                }
            }
            ERenderTargetType::FrameBuffer(options) => {
                let FrameBufferOptions { depth, .. } = options;
                match self.textures.get(&depth) {
                    Some(texture) => texture,
                    None => return,
                }
            }
        };

        let output_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        if self.is_enable_multiple_thread {
            self.draw_objects_multiple_thread(
                color_texture,
                depth_texture,
                &present_info.draw_objects,
            );
        } else {
            let mut mesh_buffers: Vec<GpuVertexBufferImp> =
                Vec::with_capacity(present_info.draw_objects.len());
            let mut g_vertex_buffers: Vec<Vec<&Buffer>> =
                Vec::with_capacity(present_info.draw_objects.len());
            let mut g_index_buffers: Vec<Option<&wgpu::Buffer>> =
                Vec::with_capacity(present_info.draw_objects.len());

            for draw_object_command in &present_info.draw_objects {
                let mut index_buffer: Option<&wgpu::Buffer> = None;
                if let Some(handle) = draw_object_command.index_buffer {
                    if let Some(buffer) = self.buffers.get(&handle) {
                        index_buffer = Some(buffer);
                    }
                }
                g_index_buffers.push(index_buffer);
            }

            for draw_object_command in &present_info.draw_objects {
                let mut vertex_buffers: Vec<&Buffer> = Vec::<&Buffer>::new();

                for vertex_buffer in &draw_object_command.vertex_buffers {
                    if let Some(vertex_buffer) = self.buffers.get(&vertex_buffer) {
                        vertex_buffers.push(vertex_buffer);
                    }
                }
                if vertex_buffers.is_empty() {
                    panic!("Vertex buffers is empty");
                }
                g_vertex_buffers.push(vertex_buffers);
            }

            for (i, draw_object_command) in present_info.draw_objects.iter().enumerate() {
                let vertex_buffers = &g_vertex_buffers[i];
                let index_buffer = &g_index_buffers[i];

                let mesh_buffer = GpuVertexBufferImp {
                    vertex_buffers: &vertex_buffers,
                    vertex_count: draw_object_command.vertex_count,
                    index_buffer: *index_buffer,
                    index_count: draw_object_command.index_count,
                    draw_type: match &draw_object_command.draw_call_type {
                        EDrawCallType::MultiDrawIndirect(multi_draw_indirect) => {
                            let indirect_buffer = self
                                .buffers
                                .get(&multi_draw_indirect.indirect_buffer_handle)
                                .unwrap();
                            crate::gpu_vertex_buffer::EDrawCallType::MultiDrawIndirect(
                                crate::gpu_vertex_buffer::MultiDrawIndirect {
                                    indirect_buffer,
                                    indirect_offset: multi_draw_indirect.indirect_offset,
                                    count: multi_draw_indirect.count,
                                },
                            )
                        }
                        EDrawCallType::Draw(draw) => crate::gpu_vertex_buffer::EDrawCallType::Draw(
                            crate::gpu_vertex_buffer::Draw {
                                instances: draw.instances.clone(),
                            },
                        ),
                    },
                };
                mesh_buffers.push(mesh_buffer);
            }

            self.draw_objects(
                color_texture.width(),
                color_texture.height(),
                &output_view,
                &depth_texture_view,
                &present_info.draw_objects,
                msaa_texture_view.as_ref(),
                msaa_depth_texture_view.as_ref(),
                mesh_buffers,
            );
        }

        (|| {
            let anti_type = &present_info.scene_viewport.anti_type;
            let EAntialiasType::FXAA(fxaa_info) = anti_type else {
                return;
            };
            let Some(fxaa_pipeline) = &self.fxaa_pipeline else {
                return;
            };
            let Some(sampler) = self.samplers.get(&fxaa_info.sampler) else {
                return;
            };
            let Some(texture) = self.textures.get(&fxaa_info.texture) else {
                return;
            };
            let _span = tracy_client::span!("fxaa");

            let queue = self.wgpu_context.get_queue();
            let device = self.wgpu_context.get_device();
            let mut command_encoder =
                device.create_command_encoder(&CommandEncoderDescriptor::default());
            command_encoder.copy_texture_to_texture(
                color_texture.as_image_copy(),
                texture.as_image_copy(),
                texture.size(),
            );
            let _ = queue.submit(vec![command_encoder.finish()]);
            let texture_view = texture.create_view(&TextureViewDescriptor::default());
            fxaa_pipeline.draw(
                device,
                queue,
                &output_view,
                vec![vec![
                    BindingResource::Sampler(sampler),
                    BindingResource::TextureView(&texture_view),
                ]],
            );
        })();

        #[cfg(feature = "renderdoc")]
        {
            if is_capture_frame {
                let device = self.wgpu_context.get_device();
                if let Some(render_doc_context) = &mut self.render_doc_context {
                    render_doc_context.stop_capture(device);
                }
            }
        }
    }

    // fn clear_buffer(
    //     &self,
    //     surface_texture_view: &TextureView,
    //     depth_texture_view: &TextureView,
    //     resolve_target: Option<&TextureView>,
    // ) {
    //     let _span = tracy_client::span!();

    //     self.attachment_pipeline.draw(
    //         self.wgpu_context.get_device(),
    //         self.wgpu_context.get_queue(),
    //         EClearType::Both(ClearAll {
    //             clear_color: ClearColor {
    //                 view: surface_texture_view,
    //                 resolve_target,
    //                 color: Color {
    //                     r: 0.0,
    //                     g: 0.0,
    //                     b: 0.0,
    //                     a: 0.0,
    //                 },
    //             },
    //             clear_depth: ClearDepth {
    //                 view: depth_texture_view,
    //             },
    //         }),
    //     );
    // }

    fn clear_shadow_depth_texture(&self, depth_texture_view: &TextureView) {
        self.attachment_pipeline.draw(
            self.wgpu_context.get_device(),
            self.wgpu_context.get_queue(),
            EClearType::Depth(ClearDepth {
                view: depth_texture_view,
            }),
        );
    }

    pub fn load_shader<K>(&mut self, shaders: HashMap<K, String>)
    where
        K: AsRef<str>,
    {
        self.shader_library
            .load_shaders_from(shaders, self.wgpu_context.get_device());
    }

    pub fn send_command(&mut self, command: RenderCommand) -> Option<RenderOutput2> {
        match command {
            RenderCommand::CreateIBLBake(command) => {
                self.process_create_iblbake_command(command);
            }
            RenderCommand::CreateTexture(create_texture_command) => {
                let device = self.wgpu_context.get_device();
                let queue = self.wgpu_context.get_queue();
                let texture = device
                    .create_texture(&create_texture_command.texture_descriptor_create_info.get());
                if let Some(init_data) = &create_texture_command.init_data {
                    queue.write_texture(
                        texture.as_image_copy(),
                        &init_data.data,
                        init_data.data_layout,
                        create_texture_command.texture_descriptor_create_info.size,
                    );
                }
                let handle = create_texture_command.handle;
                self.texture_views.insert(
                    handle,
                    texture.create_view(&TextureViewDescriptor::default()),
                );
                self.textures.insert(handle, texture);
                self.texture_descriptors.insert(
                    handle,
                    create_texture_command
                        .texture_descriptor_create_info
                        .clone(),
                );
                return Some(RenderOutput2 {
                    ty: ERenderOutputType::CreateTexture(handle),
                    error: None,
                });
            }
            RenderCommand::CreateUITexture(create_uitexture_command) => {
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
            RenderCommand::CreateBuffer(create_buffer_command) => {
                let span = tracy_client::span!("create_buffer_command");
                span.emit_text(
                    &create_buffer_command
                        .buffer_create_info
                        .label
                        .clone()
                        .unwrap_or_default(),
                );
                let device = self.wgpu_context.get_device();
                let descriptor = BufferInitDescriptor {
                    label: create_buffer_command.buffer_create_info.label.as_deref(),
                    contents: &create_buffer_command.buffer_create_info.contents,
                    usage: create_buffer_command.buffer_create_info.usage,
                };
                let new_buffer = device.create_buffer_init(&descriptor);
                let handle = create_buffer_command.handle;
                // render_output.create_buffer_handles.insert(handle);
                self.buffers.insert(handle, Arc::new(new_buffer));
                self.buffer_infos
                    .insert(handle, create_buffer_command.buffer_create_info.clone());
                return Some(RenderOutput2 {
                    ty: ERenderOutputType::CreateBuffer(handle),
                    error: None,
                });
                // self.create_buffer_commands.push(command)
            }
            RenderCommand::UpdateBuffer(update_buffer_command) => {
                let span = tracy_client::span!("update_buffer_command");
                span.emit_text(&if let Some(buffer_create_info) =
                    self.buffer_infos.get(&update_buffer_command.handle)
                {
                    buffer_create_info.label.clone().unwrap_or_default()
                } else {
                    "update_buffer_command".to_string()
                });

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
            RenderCommand::UpdateTexture(update_texture_command) => {
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

            RenderCommand::UiOutput(command) => {
                let _span = tracy_client::span!("egui render");
                let output_view = match self.surface_textures.get(&command.window_id) {
                    Some(surface_texture) => {
                        let output_view = surface_texture
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        output_view
                    }
                    None => return None,
                };
                let device = self.wgpu_context.get_device();
                let queue = self.wgpu_context.get_queue();
                self.gui_renderer
                    .render(&command, queue, device, &output_view);
            }
            RenderCommand::Resize(resize_command) => {
                //
                if resize_command.width <= 0 || resize_command.height <= 0 {
                    return None;
                }
                self.surface_size_will_change(
                    resize_command.window_id,
                    glam::uvec2(resize_command.width, resize_command.height),
                );
            }

            RenderCommand::Present(present_info) => {
                self.present(present_info);
            }
            RenderCommand::Task(task_command) => {
                let mut task = task_command.lock().unwrap();
                task(self);
            }
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
                for virtual_texture_pass in &mut self.virtual_texture_pass.values_mut() {
                    virtual_texture_pass.virtual_texture_sources.insert(
                        command.handle,
                        VirtualTextureSource::new(command.source.clone()),
                    );
                }
            }
            RenderCommand::ChangeViewMode(new_view_mode) => {
                self.skin_mesh_shading_pipeline.set_view_mode(
                    new_view_mode,
                    self.wgpu_context.get_device(),
                    &self.shader_library,
                    &mut self.base_render_pipeline_pool,
                );
                for (_, pipeline) in self.material_render_pipelines.iter_mut() {
                    pipeline.set_view_mode(
                        new_view_mode,
                        self.wgpu_context.get_device(),
                        &self.shader_library,
                        &mut self.base_render_pipeline_pool,
                    );
                }
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
                let all_options = MaterialOptions::all();
                let handle = create_render_pipeline.handle;
                let device = self.wgpu_context.get_device();
                let mut is_success = true;
                for option in all_options.iter() {
                    let name = ShaderLibrary::get_material_shader_name(handle, option);
                    let create_shader_result = self.shader_library.load_shader_from(
                        name.clone(),
                        create_render_pipeline
                            .shader_code
                            .get(option)
                            .expect(&format!("Code for {:?}", option))
                            .clone(),
                        device,
                    );
                    if is_success {
                        is_success = create_shader_result.is_ok();
                    }
                    match create_shader_result {
                        Ok(_) => {
                            log::trace!("Create material render pipeline: {}", name);
                        }
                        Err(err) => match err {
                            crate::error::Error::ShaderReflection(err, _) => {
                                log::warn!("{}", err);
                            }
                            crate::error::Error::Wgpu(err) => match err.lock().unwrap().deref() {
                                Error::OutOfMemory { .. } => {
                                    todo!()
                                }
                                Error::Validation { description, .. } => {
                                    log::trace!("Failed to create shader, {}", description);
                                }
                                Error::Internal { .. } => todo!(),
                            },
                            _ => unreachable!(),
                        },
                    }
                }
                if !is_success {
                    return None;
                }
                let current_swapchain_format = self
                    .wgpu_context
                    .get_current_swapchain_format(self.main_window_id);
                let material_render_pipeline = VariantMaterialRenderPipeline::new(
                    handle,
                    all_options,
                    device,
                    &self.shader_library,
                    &current_swapchain_format,
                    &mut self.base_render_pipeline_pool,
                );
                self.material_render_pipelines
                    .insert(handle, material_render_pipeline);
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
                            .insert(upload_prebake_ibl.key, prebake_ibl);
                    }
                    Err(err) => {
                        log::trace!("{}", err);
                    }
                }
            }
            RenderCommand::CreateVirtualTexturePass(create_virtual_texture_pass) => {
                let width = create_virtual_texture_pass.surface_size.x;
                let height = create_virtual_texture_pass.surface_size.y;
                let virtual_texture_pass = VirtualTexturePass::new(
                    self.wgpu_context.get_device(),
                    &self.shader_library,
                    false,
                    glam::uvec2(width, height),
                    create_virtual_texture_pass.settings.clone(),
                )
                .unwrap();

                self.virtual_texture_pass
                    .insert(create_virtual_texture_pass.key, virtual_texture_pass);
            }
            RenderCommand::VirtualTexturePassResize(virtual_texture_pass_resize) => {
                let Some(pass) = self
                    .virtual_texture_pass
                    .get_mut(&virtual_texture_pass_resize.key)
                else {
                    return None;
                };
                pass.change_surface_size(
                    self.wgpu_context.get_device(),
                    virtual_texture_pass_resize.surface_size,
                );
            }
            RenderCommand::ClearVirtualTexturePass(clear_virtual_texture_pass) => {
                let Some(pass) = self
                    .virtual_texture_pass
                    .get_mut(&clear_virtual_texture_pass)
                else {
                    return None;
                };
                pass.begin_new_frame(
                    self.wgpu_context.get_device(),
                    self.wgpu_context.get_queue(),
                );
            }
            RenderCommand::ClearDepthTexture(clear_depth_texture) => {
                let Some(depth_texture) = self.textures.get(&clear_depth_texture.handle) else {
                    return None;
                };
                let is_support = match depth_texture.format() {
                    TextureFormat::Depth32Float => true,
                    _ => false,
                };
                if !is_support {
                    return None;
                }
                let depth_texture_view =
                    depth_texture.create_view(&TextureViewDescriptor::default());
                self.clear_shadow_depth_texture(&depth_texture_view);
            }
            RenderCommand::CreateDefaultIBL(key) => {
                let device = self.wgpu_context.get_device();
                let prebake_ibl = PrebakeIBL::empty(device);
                match prebake_ibl {
                    Ok(prebake_ibl) => {
                        self.prebake_ibls.insert(key, prebake_ibl);
                    }
                    Err(err) => {
                        log::trace!("{}", err);
                    }
                }
            }
            RenderCommand::BuiltinShaderChanged(builtin_shader_changed) => {
                let device = self.wgpu_context.get_device();
                let load_shader_result = self.shader_library.load_shader_from(
                    builtin_shader_changed.name.clone(),
                    builtin_shader_changed.source,
                    device,
                );
                match load_shader_result {
                    Ok(_) => {
                        self.base_render_pipeline_pool
                            .invalid_shader(builtin_shader_changed.name.clone());
                    }
                    Err(err) => match err {
                        crate::error::Error::ShaderReflection(_, _) => {}
                        crate::error::Error::Wgpu(err) => match err.lock().unwrap().deref() {
                            Error::OutOfMemory { .. } => {
                                todo!()
                            }
                            Error::Validation { description, .. } => {
                                log::trace!("Failed to create shader, {}", description);
                            }
                            Error::Internal { .. } => todo!(),
                        },
                        _ => unreachable!(),
                    },
                }
                match builtin_shader_changed.name {
                    name if name
                        == crate::global_shaders::global_shader::GlobalShader::get_name(
                            &crate::global_shaders::fxaa::FXAAShader {},
                        ) =>
                    {
                        let current_swapchain_format = self
                            .wgpu_context
                            .get_current_swapchain_format(self.main_window_id);
                        self.fxaa_pipeline = Some(FXAAPipeline::new(
                            device,
                            &self.shader_library,
                            &current_swapchain_format,
                            &mut self.base_render_pipeline_pool,
                        ));
                    }
                    _ => {}
                }
            }
            RenderCommand::DestroyTextures(textures) => {
                self.textures.retain(|k, _| !textures.contains(k));
            }
            RenderCommand::ScaleChanged(info) => {
                self.gui_renderer
                    .change_scale_factor(info.window_id, info.new_factor);
            }
            RenderCommand::WindowRedrawRequestedBegin(window_id) => {
                let surface_texture = match self.wgpu_context.get_current_surface_texture(window_id)
                {
                    Ok(texture) => texture,
                    Err(err) => {
                        if err != wgpu::SurfaceError::Outdated {
                            log::warn!("{}", err);
                        }
                        return None;
                    }
                };
                let old_surface_texture = self.surface_textures.insert(window_id, surface_texture);
                assert!(old_surface_texture.is_none());
            }
            RenderCommand::WindowRedrawRequestedEnd(window_id) => {
                let surface_texture = self.surface_textures.remove(&window_id);
                if let Some(surface_texture) = surface_texture {
                    let _span = tracy_client::span!("present");
                    surface_texture.present();
                }
            }
        }
        return None;
    }

    fn vt_pass(&mut self, present_info: &PresentInfo) {
        let _span = tracy_client::span!();

        let Some(key) = &present_info.virtual_texture_pass else {
            return;
        };

        let Some(virtual_texture_pass) = &mut self.virtual_texture_pass.get_mut(&key) else {
            return;
        };

        let device = self.wgpu_context.get_device();
        let queue = self.wgpu_context.get_queue();
        for draw_object_command in &present_info.draw_objects {
            let Some(virtual_pass_set) = draw_object_command.virtual_pass_set.as_ref() else {
                continue;
            };
            let vertex_buffers: Vec<&Buffer> = virtual_pass_set
                .vertex_buffers
                .iter()
                .map(|x| self.buffers.get(x).unwrap().as_ref())
                .collect();
            let mut index_buffer: Option<&wgpu::Buffer> = None;

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
                draw_type: match &draw_object_command.draw_call_type {
                    EDrawCallType::MultiDrawIndirect(_) => {
                        todo!()
                    }
                    EDrawCallType::Draw(draw) => crate::gpu_vertex_buffer::EDrawCallType::Draw(
                        crate::gpu_vertex_buffer::Draw {
                            instances: draw.instances.clone(),
                        },
                    ),
                },
            };

            let mut group_binding_resource: Vec<Vec<BindingResource>> = vec![];
            for binding_resource in &virtual_pass_set.binding_resources {
                let mut binding_resources: Vec<BindingResource> = vec![];
                for binding_resource_type in binding_resource {
                    match binding_resource_type {
                        EBindingResource::Texture(_) => {
                            panic!()
                        }
                        EBindingResource::Constants(buffer_handle) => {
                            let buffer =
                                self.buffers.get(buffer_handle).unwrap().as_entire_binding();
                            binding_resources.push(buffer);
                        }
                        EBindingResource::Sampler(_) => {
                            panic!()
                        }
                    }
                }
                group_binding_resource.push(binding_resources);
            }

            match &draw_object_command.pipeline {
                EPipelineType::Builtin(ty) => match ty {
                    EBuiltinPipelineType::SkinMeshPhong => {
                        virtual_texture_pass.render(
                            device,
                            queue,
                            &[mesh_buffer.clone()],
                            group_binding_resource,
                            false,
                        );
                    }
                    EBuiltinPipelineType::StaticMeshPhong => {
                        virtual_texture_pass.render(
                            device,
                            queue,
                            &[mesh_buffer.clone()],
                            group_binding_resource,
                            true,
                        );
                    }
                    _ => unimplemented!(),
                },
                EPipelineType::Material(ty) => {
                    virtual_texture_pass.render(
                        device,
                        queue,
                        &[mesh_buffer.clone()],
                        group_binding_resource,
                        ty.options.is_skin == false,
                    );
                }
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

    fn _draw_object(
        &self,
        width: u32,
        height: u32,
        surface_texture_view: &wgpu::TextureView,
        depth_texture_view: &TextureView,
        draw_object_command: &DrawObject,
        resolve_target: Option<&TextureView>,
        resolve_depth_target: Option<&TextureView>,
    ) -> crate::error::Result<()> {
        let _span = tracy_client::span!();

        let _ = height;
        let _ = width;
        let device = self.wgpu_context.get_device();
        let queue = self.wgpu_context.get_queue();
        let mut vertex_buffers = Vec::<&Buffer>::new();
        let mut index_buffer: Option<&wgpu::Buffer> = None;

        for vertex_buffer in &draw_object_command.vertex_buffers {
            if let Some(vertex_buffer) = self.buffers.get(&vertex_buffer) {
                vertex_buffers.push(vertex_buffer);
            }
        }
        if vertex_buffers.is_empty() {
            return Err(crate::error::Error::Other(Some(format!(
                "Vertex buffers is empty"
            ))));
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
            draw_type: match &draw_object_command.draw_call_type {
                EDrawCallType::MultiDrawIndirect(multi_draw_indirect) => {
                    let indirect_buffer = self
                        .buffers
                        .get(&multi_draw_indirect.indirect_buffer_handle)
                        .unwrap();
                    crate::gpu_vertex_buffer::EDrawCallType::MultiDrawIndirect(
                        crate::gpu_vertex_buffer::MultiDrawIndirect {
                            indirect_buffer,
                            indirect_offset: multi_draw_indirect.indirect_offset,
                            count: multi_draw_indirect.count,
                        },
                    )
                }
                EDrawCallType::Draw(draw) => {
                    crate::gpu_vertex_buffer::EDrawCallType::Draw(crate::gpu_vertex_buffer::Draw {
                        instances: draw.instances.clone(),
                    })
                }
            },
        };

        let mut tmp_texture_views: HashMap<u64, &TextureView> = HashMap::new();

        for (group, binding_resource) in draw_object_command.binding_resources.iter().enumerate() {
            for (binding, binding_resource_type) in binding_resource.iter().enumerate() {
                match binding_resource_type {
                    EBindingResource::Texture(handle) => {
                        let mut texture_view: Option<&TextureView> = None;
                        if let Some(find_texture) = self.texture_views.get(handle) {
                            texture_view = Some(find_texture);
                        } else {
                            for (key, pass) in &self.virtual_texture_pass {
                                if key.page_table_texture_handle == *handle {
                                    texture_view = Some(pass.get_indirect_table_view());
                                    break;
                                } else if key.physical_texture_handle == *handle {
                                    texture_view = Some(pass.get_physical_texture_view());
                                    break;
                                }
                            }
                        }
                        if texture_view.is_none() {
                            for (key, value) in &self.prebake_ibls {
                                if key.brdflut_texture == *handle {
                                    texture_view = Some(value.get_brdflut_texture_view());
                                } else if key.pre_filter_cube_map_texture == *handle {
                                    texture_view =
                                        Some(value.get_pre_filter_cube_map_texture_view());
                                } else if key.irradiance_texture == *handle {
                                    texture_view = Some(value.get_irradiance_texture_view());
                                }
                            }
                        }

                        if texture_view.is_none() {
                            for (key, value) in &self.ibl_bakes {
                                if key.brdflut_texture == *handle {
                                    texture_view = Some(value.get_brdflut_texture_view());
                                } else if key.pre_filter_cube_map_texture == *handle {
                                    texture_view =
                                        Some(value.get_pre_filter_cube_map_texture_view());
                                } else if key.irradiance_texture == *handle {
                                    texture_view = Some(value.get_irradiance_texture_view());
                                }
                            }
                        }

                        let texture_view = texture_view.ok_or(crate::error::Error::Other(Some(
                            format!("{}, {}, texture view is null", group, binding),
                        )))?;
                        tmp_texture_views.insert(*handle, texture_view);
                    }
                    EBindingResource::Constants(_) => {}
                    EBindingResource::Sampler(_) => {}
                }
            }
        }

        let mut group_binding_resource: Vec<Vec<BindingResource>> = vec![];
        for (group, binding_resource) in draw_object_command.binding_resources.iter().enumerate() {
            let mut binding_resources: Vec<BindingResource> = vec![];
            for (binding, binding_resource_type) in binding_resource.iter().enumerate() {
                match binding_resource_type {
                    EBindingResource::Texture(handle) => {
                        let texture_view =
                            tmp_texture_views
                                .get(handle)
                                .ok_or(crate::error::Error::Other(Some(format!(
                                    "{}, {}, texture view is null",
                                    group, binding
                                ))))?;
                        binding_resources.push(BindingResource::TextureView(texture_view));
                    }
                    EBindingResource::Constants(buffer_handle) => {
                        let buffer = self
                            .buffers
                            .get(buffer_handle)
                            .ok_or(crate::error::Error::Other(Some(format!(
                                "{}, {}, constants is null",
                                group, binding
                            ))))?
                            .as_entire_binding();
                        binding_resources.push(buffer);
                    }
                    EBindingResource::Sampler(handle) => {
                        let sampler =
                            self.samplers
                                .get(handle)
                                .ok_or(crate::error::Error::Other(Some(format!(
                                    "{}, {}, sampler is null",
                                    group, binding
                                ))))?;
                        binding_resources.push(BindingResource::Sampler(sampler));
                    }
                }
            }
            group_binding_resource.push(binding_resources);
        }

        match &draw_object_command.pipeline {
            EPipelineType::Builtin(ty) => match ty {
                EBuiltinPipelineType::SkinMeshPhong => {
                    self.skin_mesh_shading_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &depth_texture_view,
                        &[mesh_buffer],
                        group_binding_resource,
                    );
                }
                EBuiltinPipelineType::StaticMeshPhong => {
                    self.shading_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &depth_texture_view,
                        &[mesh_buffer],
                        group_binding_resource,
                    );
                }
                EBuiltinPipelineType::Grid => {
                    self.grid_render_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        resolve_target,
                        resolve_depth_target.unwrap_or(&depth_texture_view),
                        &[mesh_buffer],
                        group_binding_resource,
                    );
                }
                EBuiltinPipelineType::MeshView => {
                    self.mesh_view_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &depth_texture_view,
                        &[mesh_buffer],
                        group_binding_resource,
                    );
                }
                EBuiltinPipelineType::MeshViewMultipleDraw => {
                    self.mesh_view_multiple_draw_pipeline.multi_draw_indirect(
                        device,
                        queue,
                        surface_texture_view,
                        &depth_texture_view,
                        &[mesh_buffer],
                        group_binding_resource,
                    );
                }
                EBuiltinPipelineType::ShadowDepthSkinMesh => unimplemented!(),
                EBuiltinPipelineType::ShadowDepthStaticMesh => unimplemented!(),
                EBuiltinPipelineType::Particle => {
                    self.particle_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &depth_texture_view,
                        &[mesh_buffer],
                        group_binding_resource,
                    );
                }
                EBuiltinPipelineType::Primitive(_) => {
                    self.primitive_render_pipeline.draw(
                        device,
                        queue,
                        surface_texture_view,
                        &depth_texture_view,
                        &[mesh_buffer],
                        group_binding_resource,
                    );
                }
            },
            EPipelineType::Material(material_pipeline) => {
                match self
                    .material_render_pipelines
                    .get(&material_pipeline.handle)
                {
                    Some(pipeline) => match pipeline.get(&material_pipeline.options) {
                        Some(render_pipeline) => {
                            render_pipeline.draw(
                                device,
                                queue,
                                surface_texture_view,
                                &depth_texture_view,
                                &[mesh_buffer],
                                group_binding_resource,
                                None,
                                None,
                            );
                        }
                        None => {
                            log::warn!("{} no match options", &material_pipeline.handle);
                        }
                    },
                    None => {
                        log::warn!("{} is not found", &material_pipeline.handle);
                    }
                }
            }
        }

        Ok(())
    }

    fn draw_objects(
        &self,
        width: u32,
        height: u32,
        surface_texture_view: &wgpu::TextureView,
        depth_texture_view: &TextureView,
        draw_object_commands: &Vec<DrawObject>,
        resolve_target: Option<&TextureView>,
        resolve_depth_target: Option<&TextureView>,
        mesh_buffers: Vec<GpuVertexBufferImp>,
    ) {
        let _ = width;
        let _ = height;
        let _span = tracy_client::span!();
        _span.emit_text(&format!("{}", draw_object_commands.len()));

        let device = self.wgpu_context.get_device();
        let queue = self.wgpu_context.get_queue();
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some(&format!("Render scene command encoder")),
        });
        let depth_stencil_attachment = RenderPassDepthStencilAttachment {
            view: resolve_depth_target.unwrap_or(depth_texture_view),
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            stencil_ops: None,
        };
        let color_attachments = vec![Some(RenderPassColorAttachment {
            ops: Operations {
                load: LoadOp::Clear(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }),
                store: StoreOp::Store,
            },
            view: resolve_target.unwrap_or(surface_texture_view),
            resolve_target: if resolve_target.is_none() {
                None
            } else {
                Some(surface_texture_view)
            },
        })];

        let mut all_bind_groups: Vec<u64> = vec![];
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(&format!("Scene render pass")),
                color_attachments: &color_attachments,
                depth_stencil_attachment: Some(depth_stencil_attachment),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            {
                for draw_object_command in draw_object_commands {
                    let mut tmp_texture_views: HashMap<u64, &TextureView> = HashMap::new();
                    {
                        for (group, binding_resource) in
                            draw_object_command.binding_resources.iter().enumerate()
                        {
                            for (binding, binding_resource_type) in
                                binding_resource.iter().enumerate()
                            {
                                match binding_resource_type {
                                    EBindingResource::Texture(handle) => {
                                        let mut texture_view: Option<&TextureView> = None;
                                        if let Some(find_texture) = self.texture_views.get(handle) {
                                            texture_view = Some(find_texture);
                                        } else {
                                            for (key, pass) in &self.virtual_texture_pass {
                                                if key.page_table_texture_handle == *handle {
                                                    texture_view =
                                                        Some(pass.get_indirect_table_view());
                                                    break;
                                                } else if key.physical_texture_handle == *handle {
                                                    texture_view =
                                                        Some(pass.get_physical_texture_view());
                                                    break;
                                                }
                                            }
                                        }
                                        if texture_view.is_none() {
                                            for (key, value) in &self.prebake_ibls {
                                                if key.brdflut_texture == *handle {
                                                    texture_view =
                                                        Some(value.get_brdflut_texture_view());
                                                } else if key.pre_filter_cube_map_texture == *handle
                                                {
                                                    texture_view = Some(
                                                        value
                                                            .get_pre_filter_cube_map_texture_view(),
                                                    );
                                                } else if key.irradiance_texture == *handle {
                                                    texture_view =
                                                        Some(value.get_irradiance_texture_view());
                                                }
                                            }
                                        }

                                        if texture_view.is_none() {
                                            for (key, value) in &self.ibl_bakes {
                                                if key.brdflut_texture == *handle {
                                                    texture_view =
                                                        Some(value.get_brdflut_texture_view());
                                                } else if key.pre_filter_cube_map_texture == *handle
                                                {
                                                    texture_view = Some(
                                                        value
                                                            .get_pre_filter_cube_map_texture_view(),
                                                    );
                                                } else if key.irradiance_texture == *handle {
                                                    texture_view =
                                                        Some(value.get_irradiance_texture_view());
                                                }
                                            }
                                        }

                                        let texture_view = texture_view
                                            .ok_or(crate::error::Error::Other(Some(format!(
                                                "{}, {}, texture view is null",
                                                group, binding
                                            ))))
                                            .expect("Texture should not be null");
                                        tmp_texture_views.insert(*handle, texture_view);
                                    }
                                    EBindingResource::Constants(_) => {}
                                    EBindingResource::Sampler(_) => {}
                                }
                            }
                        }
                    }

                    let mut hasher = std::hash::DefaultHasher::new();
                    let mut group_binding_resource: Vec<Vec<BindingResource>> =
                        Vec::with_capacity(draw_object_command.binding_resources.len());
                    for (group, binding_resource) in
                        draw_object_command.binding_resources.iter().enumerate()
                    {
                        let mut binding_resources: Vec<BindingResource> =
                            Vec::with_capacity(binding_resource.len());
                        for (binding, binding_resource_type) in binding_resource.iter().enumerate()
                        {
                            match binding_resource_type {
                                EBindingResource::Texture(handle) => {
                                    handle.hash(&mut hasher);
                                    let texture_view = tmp_texture_views
                                        .get(handle)
                                        .ok_or(crate::error::Error::Other(Some(format!(
                                            "{}, {}, texture view is null",
                                            group, binding
                                        ))))
                                        .expect("Texture should not be null");
                                    binding_resources
                                        .push(BindingResource::TextureView(texture_view));
                                }
                                EBindingResource::Constants(buffer_handle) => {
                                    buffer_handle.hash(&mut hasher);
                                    let buffer = self
                                        .buffers
                                        .get(buffer_handle)
                                        .ok_or(crate::error::Error::Other(Some(format!(
                                            "{}, {}, constants is null",
                                            group, binding
                                        ))))
                                        .expect("Texture should not be null")
                                        .as_entire_binding();
                                    binding_resources.push(buffer);
                                }
                                EBindingResource::Sampler(handle) => {
                                    handle.hash(&mut hasher);
                                    let sampler = self
                                        .samplers
                                        .get(handle)
                                        .ok_or(crate::error::Error::Other(Some(format!(
                                            "{}, {}, sampler is null",
                                            group, binding
                                        ))))
                                        .expect("Texture should not be null");
                                    binding_resources.push(BindingResource::Sampler(sampler));
                                }
                            }
                        }
                        group_binding_resource.push(binding_resources);
                    }

                    let pipeline = match &draw_object_command.pipeline {
                        EPipelineType::Builtin(ty) => match ty {
                            EBuiltinPipelineType::SkinMeshPhong => Some(
                                self.skin_mesh_shading_pipeline
                                    .base_render_pipeline
                                    .as_ref(),
                            ),
                            EBuiltinPipelineType::StaticMeshPhong => {
                                Some(&self.shading_pipeline.base_render_pipeline)
                            }
                            EBuiltinPipelineType::Grid => {
                                Some(self.grid_render_pipeline.base_render_pipeline.as_ref())
                            }
                            EBuiltinPipelineType::MeshView => {
                                Some(self.mesh_view_pipeline.base_render_pipeline.as_ref())
                            }
                            EBuiltinPipelineType::MeshViewMultipleDraw => Some(
                                self.mesh_view_multiple_draw_pipeline
                                    .base_render_pipeline
                                    .as_ref(),
                            ),
                            EBuiltinPipelineType::ShadowDepthSkinMesh => unimplemented!(),
                            EBuiltinPipelineType::ShadowDepthStaticMesh => unimplemented!(),
                            EBuiltinPipelineType::Particle => {
                                Some(self.particle_pipeline.base_render_pipeline.as_ref())
                            }
                            EBuiltinPipelineType::Primitive(primitive) => {
                                let base_render_pipeline = self
                                    .primitive_render_pipeline
                                    .get_base_render_pipeline(primitive.as_ref());
                                base_render_pipeline
                                // Some(base_render_pipeline)
                                // Some(self.primitive_render_pipeline.base_render_pipeline.as_ref())
                            }
                        },
                        EPipelineType::Material(material_pipeline) => {
                            match self
                                .material_render_pipelines
                                .get(&material_pipeline.handle)
                            {
                                Some(pipeline) => {
                                    match pipeline.get(&material_pipeline.options) {
                                        Some(render_pipeline) => {
                                            Some(render_pipeline.base_render_pipeline.as_ref())
                                        }
                                        None => {
                                            log::warn!(
                                                "{} no match options",
                                                &material_pipeline.handle
                                            );
                                            None
                                            // panic!("{} no match options", &material_pipeline.handle);
                                        }
                                    }
                                }
                                None => {
                                    log::warn!("{} is not found", &material_pipeline.handle);
                                    None
                                    // panic!("{} is not found", &material_pipeline.handle);
                                }
                            }
                        }
                    };

                    if let Some(pipeline) = pipeline {
                        let cache_key = hasher.finish();
                        if !self.bind_groups_collection.contains_key(&cache_key) {
                            self.bind_groups_collection.insert(
                                cache_key,
                                Arc::new(pipeline.make_bind_groups_binding_resources(
                                    device,
                                    group_binding_resource,
                                )),
                            );
                        }

                        all_bind_groups.push(cache_key);
                    } else {
                        all_bind_groups.push(0);
                    }
                }
            }
            for (i, draw_object_command) in draw_object_commands.iter().enumerate() {
                let pipeline = match &draw_object_command.pipeline {
                    EPipelineType::Builtin(ty) => match ty {
                        EBuiltinPipelineType::SkinMeshPhong => Some(
                            self.skin_mesh_shading_pipeline
                                .base_render_pipeline
                                .as_ref(),
                        ),
                        EBuiltinPipelineType::StaticMeshPhong => {
                            Some(&self.shading_pipeline.base_render_pipeline)
                        }
                        EBuiltinPipelineType::Grid => {
                            Some(self.grid_render_pipeline.base_render_pipeline.as_ref())
                        }
                        EBuiltinPipelineType::MeshView => {
                            Some(self.mesh_view_pipeline.base_render_pipeline.as_ref())
                        }
                        EBuiltinPipelineType::MeshViewMultipleDraw => Some(
                            self.mesh_view_multiple_draw_pipeline
                                .base_render_pipeline
                                .as_ref(),
                        ),
                        EBuiltinPipelineType::ShadowDepthSkinMesh => unimplemented!(),
                        EBuiltinPipelineType::ShadowDepthStaticMesh => unimplemented!(),
                        EBuiltinPipelineType::Particle => {
                            Some(self.particle_pipeline.base_render_pipeline.as_ref())
                        }
                        EBuiltinPipelineType::Primitive(primitive) => {
                            let base_render_pipeline = self
                                .primitive_render_pipeline
                                .get_base_render_pipeline(primitive.as_ref());
                            base_render_pipeline
                            // Some(self.primitive_render_pipeline.base_render_pipeline.as_ref())
                        }
                    },
                    EPipelineType::Material(material_pipeline) => {
                        match self
                            .material_render_pipelines
                            .get(&material_pipeline.handle)
                        {
                            Some(pipeline) => match pipeline.get(&material_pipeline.options) {
                                Some(render_pipeline) => {
                                    Some(render_pipeline.base_render_pipeline.as_ref())
                                }
                                None => {
                                    log::warn!("{} no match options", &material_pipeline.handle);
                                    None
                                    // panic!("{} no match options", &material_pipeline.handle);
                                }
                            },
                            None => {
                                log::warn!("{} is not found", &material_pipeline.handle);
                                None
                                // panic!("{} is not found", &material_pipeline.handle);
                            }
                        }
                    }
                };
                let Some(pipeline) = pipeline else {
                    continue;
                };

                let bind_groups = self
                    .bind_groups_collection
                    .get(&all_bind_groups[i])
                    .expect("Too many draw objects");

                pipeline.draw_with_pass(
                    &mut render_pass,
                    &bind_groups,
                    &vec![mesh_buffers[i].clone()],
                    draw_object_command.scissor_rect,
                    draw_object_command.viewport.clone(),
                    draw_object_command.debug_group_label.as_deref(),
                );
            }
        }
        queue.submit(vec![encoder.finish()]);

        // for draw_object_command in draw_object_commands {
        //     let draw_result = self.draw_object(
        //         width,
        //         height,
        //         surface_texture_view,
        //         depth_texture_view,
        //         draw_object_command,
        //         resolve_target,
        //         resolve_depth_target,
        //     );
        //     match draw_result {
        //         Ok(_) => {}
        //         Err(err) => {
        //             log::trace!("{}, {}", draw_object_command.id, err);
        //         }
        //     }
        // }
    }

    fn draw_objects_multiple_thread(
        &self,
        surface_texture: &wgpu::Texture,
        depth_texture: &wgpu::Texture,
        draw_object_commands: &Vec<DrawObject>,
    ) {
        let span = tracy_client::span!();
        span.emit_text(&format!("{}", draw_object_commands.len()));
        let device = self.get_device();
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut is_finish = draw_object_commands.len();

        struct TaskResult {
            index: usize,
            command_buffer: wgpu::CommandBuffer,
        }

        for (index, draw_object_command) in draw_object_commands.iter().enumerate() {
            let sender = sender.clone();

            let render_pipeline = match &draw_object_command.pipeline {
                EPipelineType::Builtin(ty) => match ty {
                    EBuiltinPipelineType::Grid => {
                        self.grid_render_pipeline.base_render_pipeline.clone()
                    }
                    EBuiltinPipelineType::SkinMeshPhong => {
                        self.grid_render_pipeline.base_render_pipeline.clone()
                    }
                    _ => unimplemented!(),
                },
                EPipelineType::Material(material_pipeline) => {
                    match self
                        .material_render_pipelines
                        .get(&material_pipeline.handle)
                    {
                        Some(pipeline) => match pipeline.get(&material_pipeline.options) {
                            Some(render_pipeline) => render_pipeline.base_render_pipeline.clone(),
                            None => {
                                log::warn!("{} no match options", &material_pipeline.handle);
                                continue;
                            }
                        },
                        None => {
                            log::warn!("{} is not found", &material_pipeline.handle);
                            continue;
                        }
                    }
                }
            };

            let tag = render_pipeline.get_tag().to_string();
            let slots = render_pipeline.slots;
            let encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some(&format!("{} command encoder", &tag)),
            });

            let mut tmp_texture_views: HashMap<u64, &TextureView> = HashMap::new();

            for (group, binding_resource) in
                draw_object_command.binding_resources.iter().enumerate()
            {
                for (binding, binding_resource_type) in binding_resource.iter().enumerate() {
                    match binding_resource_type {
                        EBindingResource::Texture(handle) => {
                            let mut texture_view: Option<&TextureView> = None;
                            if let Some(find_texture) = self.texture_views.get(handle) {
                                texture_view = Some(find_texture);
                            } else {
                                for (key, pass) in &self.virtual_texture_pass {
                                    if key.page_table_texture_handle == *handle {
                                        texture_view = Some(pass.get_indirect_table_view());
                                        break;
                                    } else if key.physical_texture_handle == *handle {
                                        texture_view = Some(pass.get_physical_texture_view());
                                        break;
                                    }
                                }
                            }
                            if texture_view.is_none() {
                                for (key, value) in &self.prebake_ibls {
                                    if key.brdflut_texture == *handle {
                                        texture_view = Some(value.get_brdflut_texture_view());
                                    } else if key.pre_filter_cube_map_texture == *handle {
                                        texture_view =
                                            Some(value.get_pre_filter_cube_map_texture_view());
                                    } else if key.irradiance_texture == *handle {
                                        texture_view = Some(value.get_irradiance_texture_view());
                                    }
                                }
                            }

                            if texture_view.is_none() {
                                for (key, value) in &self.ibl_bakes {
                                    if key.brdflut_texture == *handle {
                                        texture_view = Some(value.get_brdflut_texture_view());
                                    } else if key.pre_filter_cube_map_texture == *handle {
                                        texture_view =
                                            Some(value.get_pre_filter_cube_map_texture_view());
                                    } else if key.irradiance_texture == *handle {
                                        texture_view = Some(value.get_irradiance_texture_view());
                                    }
                                }
                            }

                            let texture_view = texture_view
                                .ok_or(crate::error::Error::Other(Some(format!(
                                    "{}, {}, texture view is null",
                                    group, binding
                                ))))
                                .unwrap();
                            tmp_texture_views.insert(*handle, texture_view);
                        }
                        EBindingResource::Constants(_) => {}
                        EBindingResource::Sampler(_) => {}
                    }
                }
            }
            let mut group_binding_resource: Vec<Vec<BindingResource>> = vec![];
            for (group, binding_resource) in
                draw_object_command.binding_resources.iter().enumerate()
            {
                let mut binding_resources: Vec<BindingResource> = vec![];
                for (binding, binding_resource_type) in binding_resource.iter().enumerate() {
                    match binding_resource_type {
                        EBindingResource::Texture(handle) => {
                            let texture_view = tmp_texture_views
                                .get(handle)
                                .ok_or(crate::error::Error::Other(Some(format!(
                                    "{}, {}, texture view is null",
                                    group, binding
                                ))))
                                .unwrap();
                            binding_resources.push(BindingResource::TextureView(texture_view));
                        }
                        EBindingResource::Constants(buffer_handle) => {
                            let buffer = self
                                .buffers
                                .get(buffer_handle)
                                .ok_or(crate::error::Error::Other(Some(format!(
                                    "{}, {}, constants is null",
                                    group, binding
                                ))))
                                .unwrap()
                                .as_entire_binding();
                            binding_resources.push(buffer);
                        }
                        EBindingResource::Sampler(handle) => {
                            let sampler = self
                                .samplers
                                .get(handle)
                                .ok_or(crate::error::Error::Other(Some(format!(
                                    "{}, {}, sampler is null",
                                    group, binding
                                ))))
                                .unwrap();
                            binding_resources.push(BindingResource::Sampler(sampler));
                        }
                    }
                }
                group_binding_resource.push(binding_resources);
            }

            let entries: Vec<Vec<BindGroupEntry>> = group_binding_resource
                .iter()
                .map(|x| {
                    x.iter()
                        .enumerate()
                        .map(|(binding, resource)| wgpu::BindGroupEntry {
                            binding: binding as u32,
                            resource: resource.clone(),
                        })
                        .collect()
                })
                .collect();
            let mut bind_groups: Vec<BindGroup> = Vec::new();
            for (entry_vec, bind_group_layout) in entries
                .iter()
                .zip(render_pipeline.bind_group_layouts.iter())
            {
                let bind_group = device.create_bind_group(&BindGroupDescriptor {
                    layout: &bind_group_layout,
                    entries: &entry_vec,
                    label: Some(&format!("{} bind group", &tag)),
                });
                bind_groups.push(bind_group);
            }
            let mut vertex_buffers = Vec::<Arc<Buffer>>::new();
            let mut index_buffer: Option<Arc<Buffer>> = None;

            for vertex_buffer in &draw_object_command.vertex_buffers {
                if let Some(vertex_buffer) = self.buffers.get(&vertex_buffer) {
                    vertex_buffers.push(vertex_buffer.clone());
                }
            }
            if vertex_buffers.is_empty() {
                panic!()
            }
            if let Some(handle) = draw_object_command.index_buffer {
                if let Some(buffer) = self.buffers.get(&handle) {
                    index_buffer = Some(buffer.clone());
                }
            }
            let multiple_threading_mesh_buffer =
                crate::gpu_vertex_buffer::MultipleThreadingGpuVertexBufferImp {
                    vertex_buffers,
                    vertex_count: draw_object_command.vertex_count,
                    index_buffer,
                    index_count: draw_object_command.index_count,
                    draw_type: match &draw_object_command.draw_call_type {
                        EDrawCallType::MultiDrawIndirect(multi_draw_indirect) => {
                            let indirect_buffer = self
                                .buffers
                                .get(&multi_draw_indirect.indirect_buffer_handle)
                                .unwrap()
                                .clone();
                            crate::gpu_vertex_buffer::EMultipleThreadingDrawCallType::MultiDrawIndirect(
                            crate::gpu_vertex_buffer::MutilpleThreadingMultiDrawIndirect {
                                indirect_buffer,
                                indirect_offset: multi_draw_indirect.indirect_offset,
                                count: multi_draw_indirect.count,
                            },
                        )
                        }
                        EDrawCallType::Draw(draw) => {
                            crate::gpu_vertex_buffer::EMultipleThreadingDrawCallType::Draw(
                                crate::gpu_vertex_buffer::Draw {
                                    instances: draw.instances.clone(),
                                },
                            )
                        }
                    },
                };
            let output_view = surface_texture.create_view(&TextureViewDescriptor::default());
            let multiple_threading_color_attachments = [
                crate::base_render_pipeline::MultipleThreadingColorAttachment {
                    color_ops: None,
                    view: output_view,
                    resolve_target: None,
                },
            ];
            let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

            ThreadPool::multithreaded_rendering().spawn({
                move || {
                    let mesh_buffer = GpuVertexBufferImp {
                        vertex_buffers: &multiple_threading_mesh_buffer
                            .vertex_buffers
                            .iter()
                            .map(|x| x.as_ref())
                            .collect::<Vec<&wgpu::Buffer>>(),
                        vertex_count: multiple_threading_mesh_buffer.vertex_count,
                        index_buffer: match &multiple_threading_mesh_buffer.index_buffer {
                            Some(x) => Some(x.as_ref()),
                            None => None,
                        },
                        index_count: multiple_threading_mesh_buffer.index_count,
                        draw_type: multiple_threading_mesh_buffer.draw_type.to_local(),
                    };
                    let mut color_attachments: Vec<crate::base_render_pipeline::ColorAttachment> =
                        vec![];
                    for x in &multiple_threading_color_attachments {
                        color_attachments.push(crate::base_render_pipeline::ColorAttachment {
                            color_ops: x.color_ops,
                            view: &x.view,
                            resolve_target: x.resolve_target.as_ref(),
                        });
                    }

                    let command_buffer =
                        crate::base_render_pipeline::BaseRenderPipeline::draw_multiple_threading(
                            &render_pipeline.render_pipeline,
                            tag,
                            slots,
                            bind_groups,
                            encoder,
                            &vec![mesh_buffer],
                            &color_attachments,
                            None,
                            None,
                            Some(&depth_view),
                            None,
                            None,
                        );
                    let _ = sender.send(TaskResult {
                        index,
                        command_buffer,
                    });
                }
            });
        }

        let mut task_results = vec![];
        let mut command_buffers = vec![];

        while let Ok(task_result) = receiver.recv() {
            task_results.push(task_result);
            is_finish -= 1;
            if is_finish == 0 {
                break;
            }
        }

        task_results.sort_by(|left, right| right.index.cmp(&left.index));

        while let Some(task_result) = task_results.pop() {
            command_buffers.push(task_result.command_buffer);
        }

        let queue = self.get_queue();
        queue.submit(command_buffers);
    }

    fn set_settings(&mut self, settings: RenderSettings) {
        if settings.virtual_texture_setting.is_enable {
            for (_, v) in &mut self.virtual_texture_pass {
                v.set_settings(settings.virtual_texture_setting.clone());
            }
        } else {
            self.virtual_texture_pass.clear();
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
    }

    pub fn get_device(&self) -> &wgpu::Device {
        self.wgpu_context.get_device()
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        self.wgpu_context.get_queue()
    }

    pub fn get_shader_library(&self) -> &ShaderLibrary {
        &self.shader_library
    }

    fn shadow_for_draw_objects(
        &mut self,
        draw_objects: &[DrawObject],
        depth_texture_handle: Option<TextureHandle>,
    ) {
        let Some(shadow_pipilines) = self.shadow_pipilines.as_mut() else {
            return;
        };
        let Some(depth_texture_handle) = depth_texture_handle else {
            return;
        };

        let Some(depth_view) = self.texture_views.get(&depth_texture_handle) else {
            return;
        };
        let _span = tracy_client::span!();

        let mut vertex_bufferss: Vec<Vec<&Buffer>> = Vec::with_capacity(draw_objects.len());
        let mut index_bufferss: Vec<Option<&Buffer>> = Vec::with_capacity(draw_objects.len());
        let mut bind_groupss: Vec<u64> = Vec::with_capacity(draw_objects.len());
        let mut mesh_buffers: Vec<GpuVertexBufferImp<'_>> = Vec::with_capacity(draw_objects.len());
        let device = self.wgpu_context.get_device();

        for draw_object in draw_objects {
            let Some(shadow_mapping) = &draw_object.shadow_mapping else {
                vertex_bufferss.push(vec![]);
                index_bufferss.push(None);
                bind_groupss.push(0);
                continue;
            };
            let base_render_pipeline = if shadow_mapping.is_skin {
                shadow_pipilines
                    .depth_skin_pipeline
                    .base_render_pipeline
                    .clone()
            } else {
                shadow_pipilines.depth_pipeline.base_render_pipeline.clone()
            };
            let vertex_buffers: Vec<&Buffer> = shadow_mapping
                .vertex_buffers
                .iter()
                .flat_map(|handle| self.buffers.get(handle))
                .map(|x| x.as_ref())
                .collect();
            vertex_bufferss.push(vertex_buffers);
            let index_buffer: Option<&Buffer> = draw_object
                .index_buffer
                .map(|x| self.buffers.get(&x))
                .map(|x| x.map(|x| x.as_ref()))
                .flatten();
            index_bufferss.push(index_buffer);

            let mut hasher = std::hash::DefaultHasher::new();
            let mut group_binding_resources: Vec<Vec<BindingResource>> = vec![];
            for (group, group_binding_resource) in
                shadow_mapping.binding_resources.iter().enumerate()
            {
                let mut binding_resources: Vec<BindingResource> = vec![];
                for (binding, binding_resource) in group_binding_resource.iter().enumerate() {
                    match binding_resource {
                        EBindingResource::Texture(_) => panic!(),
                        EBindingResource::Constants(buffer_handle) => {
                            buffer_handle.hash(&mut hasher);
                            let _ = self
                                .buffers
                                .get(buffer_handle)
                                .ok_or(crate::error::Error::Other(Some(format!(
                                    "{}, {}, constants is null",
                                    group, binding
                                ))))
                                .map(|x| x.as_entire_binding())
                                .and_then(|x| Ok(binding_resources.push(x)));
                        }
                        EBindingResource::Sampler(_) => panic!(),
                    }
                }
                group_binding_resources.push(binding_resources);
            }
            let cache_key = hasher.finish();
            if !self.bind_groups_collection.contains_key(&cache_key) {
                self.bind_groups_collection.insert(
                    cache_key,
                    Arc::new(
                        base_render_pipeline
                            .make_bind_groups_binding_resources(device, group_binding_resources),
                    ),
                );
            }
            bind_groupss.push(cache_key);
        }

        for (i, draw_object) in draw_objects.iter().enumerate() {
            let vertex_buffers = &vertex_bufferss[i];
            let index_buffer = index_bufferss[i];
            let mesh_buffer = GpuVertexBufferImp {
                vertex_buffers: &vertex_buffers,
                vertex_count: draw_object.vertex_count,
                index_buffer,
                index_count: draw_object.index_count,
                draw_type: match &draw_object.draw_call_type {
                    EDrawCallType::MultiDrawIndirect(multi_draw_indirect) => {
                        let indirect_buffer = self
                            .buffers
                            .get(&multi_draw_indirect.indirect_buffer_handle)
                            .unwrap();
                        crate::gpu_vertex_buffer::EDrawCallType::MultiDrawIndirect(
                            crate::gpu_vertex_buffer::MultiDrawIndirect {
                                indirect_buffer,
                                indirect_offset: multi_draw_indirect.indirect_offset,
                                count: multi_draw_indirect.count,
                            },
                        )
                    }
                    EDrawCallType::Draw(draw) => crate::gpu_vertex_buffer::EDrawCallType::Draw(
                        crate::gpu_vertex_buffer::Draw {
                            instances: draw.instances.clone(),
                        },
                    ),
                },
            };
            mesh_buffers.push(mesh_buffer);
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some(&format!("{} command encoder", "Shadow")),
        });
        let render_pass_color_attachments: Vec<Option<RenderPassColorAttachment>> = Vec::new();
        let depth_stencil_attachment = RenderPassDepthStencilAttachment {
            view: &depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            stencil_ops: None,
        };

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(&format!("Shadow pass")),
                color_attachments: &render_pass_color_attachments,
                occlusion_query_set: None,
                depth_stencil_attachment: Some(depth_stencil_attachment),
                timestamp_writes: None,
            });

            for (i, bind_groups_key) in bind_groupss.drain(..).enumerate() {
                let draw_object = &draw_objects[i];
                let Some(shadow_mapping) = &draw_object.shadow_mapping else {
                    continue;
                };
                let base_render_pipeline = if shadow_mapping.is_skin {
                    shadow_pipilines
                        .depth_skin_pipeline
                        .base_render_pipeline
                        .clone()
                } else {
                    shadow_pipilines.depth_pipeline.base_render_pipeline.clone()
                };

                let bind_groups = self
                    .bind_groups_collection
                    .get(&bind_groups_key)
                    .expect("Should not be null.");
                base_render_pipeline.draw_with_pass(
                    &mut render_pass,
                    &bind_groups,
                    &vec![mesh_buffers[i].clone()],
                    None,
                    None,
                    draw_object.debug_group_label.as_deref(),
                );
            }
        }

        self.wgpu_context.get_queue().submit(Some(encoder.finish()));
    }

    pub fn insert_new_texture(&mut self, handle: TextureHandle, texture: Texture) {
        self.textures.insert(handle, texture);
    }

    pub fn get_textures(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(&handle)
    }

    pub fn get_base_compute_pipeline_pool(&self) -> &BaseComputePipelinePool {
        &self.base_compute_pipeline_pool
    }

    pub fn get_texture_descriptor(
        &self,
        handle: TextureHandle,
    ) -> Option<&TextureDescriptorCreateInfo> {
        let descriptor = self.texture_descriptors.get(&handle);
        descriptor
    }

    fn light_culling_pass(&mut self, scene_light: &SceneLight) {
        if scene_light.point_light_shapes.is_empty() {
            self.light_culling_no_lights_fallback_pass(scene_light);
            return;
        }
        let device = self.wgpu_context.get_device();
        let queue = self.wgpu_context.get_queue();
        let get_result = self.light_culling.get_or_add(
            device,
            10 * 10 * 10,
            scene_light.point_light_shapes.len(),
        );
        self.light_culling_compute_pipeline.execute_out(
            device,
            queue,
            &scene_light.point_light_shapes,
            &scene_light.frustum,
            &get_result.cluster_lights_buffer,
            &get_result.cluster_light_indices_buffer,
            glam::uvec3(10, 10, 10),
        );

        self.buffers.insert(
            scene_light.cluster_lights_placeholder,
            get_result.cluster_lights_buffer,
        );
        self.buffers.insert(
            scene_light.cluster_light_indices_placeholder,
            get_result.cluster_light_indices_buffer,
        );
    }

    fn light_culling_no_lights_fallback_pass(&mut self, scene_light: &SceneLight) {
        let device = self.wgpu_context.get_device();
        let cluster_lights_placeholder = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (size_of::<u32>() * 1) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let cluster_light_indices_placeholder = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<crate::constants::ClusterLightIndex>() as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        self.buffers.insert(
            scene_light.cluster_lights_placeholder,
            Arc::new(cluster_lights_placeholder),
        );
        self.buffers.insert(
            scene_light.cluster_light_indices_placeholder,
            Arc::new(cluster_light_indices_placeholder),
        );
    }
}
