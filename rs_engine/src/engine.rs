use crate::build_built_in_resouce_url;
use crate::camera::Camera;
use crate::console_cmd::ConsoleCmd;
use crate::content::content_file_type::EContentFileType;
use crate::default_textures::DefaultTextures;
use crate::drawable::{
    EDrawObjectType, MaterialDrawObject, SkinMeshDrawObject, StaticMeshDrawObject,
    StaticMeshMaterialDrawObject,
};
use crate::error::Result;
use crate::handle::{EGUITextureHandle, TextureHandle};
use crate::player_viewport::PlayerViewport;
use crate::render_thread_mode::ERenderThreadMode;
use crate::{logger::Logger, resource_manager::ResourceManager};
use rs_artifact::artifact::ArtifactReader;
use rs_artifact::content_type::EContentType;
use rs_artifact::resource_info::ResourceInfo;
use rs_artifact::resource_type::EResourceType;
use rs_audio::audio_device::AudioDevice;
use rs_core_minimal::settings::Settings;
use rs_foundation::new::{
    MultipleThreadMut, MultipleThreadMutType, SingleThreadMut, SingleThreadMutType,
};
use rs_render::bake_info::BakeInfo;
use rs_render::command::{
    BufferCreateInfo, ClearDepthTexture, CreateBuffer, CreateIBLBake, CreateMaterialRenderPipeline,
    CreateSampler, CreateTexture, CreateUITexture, CreateVirtualTexture, CreateVirtualTexturePass,
    EBindingResource, InitTextureData, PresentInfo, RenderCommand, TextureDescriptorCreateInfo,
    UpdateBuffer, UploadPrebakeIBL, VirtualTexturePassKey,
};
use rs_render::egui_render::EGUIRenderOutput;
use rs_render::global_uniform::{self};
use rs_render::renderer::Renderer;
use rs_render::sdf2d_generator;
use rs_render::view_mode::EViewModeType;
use rs_render::virtual_texture_source::TVirtualTextureSource;
use rs_render_types::MaterialOptions;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VirtualPassHandle {
    pub physical_texture_handle: crate::handle::TextureHandle,
    pub page_table_texture_handle: crate::handle::TextureHandle,
}

impl VirtualPassHandle {
    pub fn new() -> VirtualPassHandle {
        let rm = ResourceManager::default();
        VirtualPassHandle {
            physical_texture_handle: rm
                .next_texture(build_built_in_resouce_url("Virtual_PhysicalTexture").unwrap()),
            page_table_texture_handle: rm
                .next_texture(build_built_in_resouce_url("Virtual_PageTableTexture").unwrap()),
        }
    }

    pub fn key(&self) -> VirtualTexturePassKey {
        VirtualTexturePassKey {
            physical_texture_handle: *self.physical_texture_handle,
            page_table_texture_handle: *self.page_table_texture_handle,
        }
    }
}

pub struct Engine {
    render_thread_mode: ERenderThreadMode,
    resource_manager: ResourceManager,
    logger: Logger,
    draw_object_id: u32,
    settings: Settings,
    game_time: std::time::Instant,
    game_time_sec: f32,
    virtual_texture_source_infos: SingleThreadMutType<
        HashMap<url::Url, MultipleThreadMutType<Box<dyn TVirtualTextureSource>>>,
    >,
    console_cmds: SingleThreadMutType<HashMap<String, SingleThreadMutType<ConsoleCmd>>>,
    pub content_files: HashMap<url::Url, EContentFileType>,
    main_window_id: isize,
    default_textures: DefaultTextures,
    virtual_pass_handle: Option<VirtualPassHandle>,
    _audio_device: Option<AudioDevice>,
}

impl Engine {
    pub fn new<W>(
        window_id: isize,
        window: &W,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
        logger: Logger,
        mut artifact_reader: Option<ArtifactReader>,
        mut shaders: HashMap<String, String>,
    ) -> Result<Engine>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let _span = tracy_client::span!();

        let settings: Settings;
        if let Some(artifact_reader) = &mut artifact_reader {
            settings = artifact_reader.get_artifact_file_header().settings.clone();
            log::trace!("Load settings: {:?}", settings);
            artifact_reader.check_assets().expect("Valid");
        } else {
            #[cfg(feature = "editor")]
            {
                settings = Self::read_or_create_editor_settings().unwrap_or(Settings::default());
            }
            #[cfg(not(feature = "editor"))]
            {
                settings = Settings::default();
            }
            log::trace!("Use default settings: {:?}", settings);
        }

        #[cfg(target_os = "android")]
        (|| {
            if settings.render_setting.get_backends_platform()
                == rs_core_minimal::settings::Backends::DX12
            {
                panic!("Not supported render backend on android platform.");
            }
        })();

        let resource_manager = ResourceManager::default();
        resource_manager.set_artifact_reader(artifact_reader);
        resource_manager.load_static_meshs();

        for shader_source_code in resource_manager.get_all_shader_source_codes() {
            shaders.insert(shader_source_code.name, shader_source_code.code);
        }

        let renderer = Renderer::from_window(
            window_id,
            window,
            surface_width,
            surface_height,
            scale_factor,
            shaders,
            settings.render_setting.clone(),
        )
        .map_err(|err| crate::error::Error::RendererError(err))?;

        let mut render_thread_mode = ERenderThreadMode::from(renderer, true);
        let mut virtual_pass_handle: Option<VirtualPassHandle> = None;
        if settings.render_setting.virtual_texture_setting.is_enable {
            let handle = VirtualPassHandle::new();
            render_thread_mode.send_command(RenderCommand::CreateVirtualTexturePass(
                CreateVirtualTexturePass {
                    key: handle.key(),
                    surface_size: glam::uvec2(surface_width, surface_height),
                    settings: settings.render_setting.virtual_texture_setting.clone(),
                },
            ));
            virtual_pass_handle = Some(handle);
        }

        let global_constants_handle = resource_manager.next_buffer();
        let global_constants = global_uniform::Constants::default();
        let command = RenderCommand::CreateBuffer(CreateBuffer {
            handle: *global_constants_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some("Global.Constants".to_string()),
                contents: rs_foundation::cast_to_raw_buffer(&vec![global_constants]).to_vec(),
                usage: wgpu::BufferUsages::all(),
            },
        });
        render_thread_mode.send_command(command);

        let global_sampler_handle = resource_manager.next_sampler();
        let command = RenderCommand::CreateSampler(CreateSampler {
            handle: *global_sampler_handle,
            sampler_descriptor: wgpu::SamplerDescriptor::default(),
        });
        render_thread_mode.send_command(command);

        let mut camera = Camera::default(surface_width, surface_height);
        camera.set_world_location(glam::vec3(0.0, 10.0, 20.0));

        let draw_object_id: u32 = 0;

        let default_textures = DefaultTextures::new(ResourceManager::default());
        default_textures.create(&mut render_thread_mode);

        let mut audio_device =
            AudioDevice::new().map_err(|err| crate::error::Error::AudioError(err))?;
        audio_device
            .play()
            .map_err(|err| crate::error::Error::AudioError(err))?;
        let virtual_texture_source_infos = SingleThreadMut::new(HashMap::new());
        let mut engine = Engine {
            render_thread_mode,
            resource_manager,
            logger,

            settings: settings.clone(),
            draw_object_id,
            game_time: std::time::Instant::now(),
            game_time_sec: 0.0,

            virtual_texture_source_infos: virtual_texture_source_infos.clone(),
            console_cmds: SingleThreadMut::new(HashMap::new()),
            content_files: Self::collect_content_files(),
            main_window_id: window_id,
            default_textures,
            virtual_pass_handle,
            // shadow_depth_texture_handle: None,
            _audio_device: Some(audio_device),
        };

        ResourceManager::default().create_builtin_resources(&mut engine);

        Ok(engine)
    }

    #[cfg(feature = "editor")]
    fn read_or_create_editor_settings() -> crate::error::Result<Settings> {
        let path = std::env::current_dir().map_err(|err| crate::error::Error::IO(err, None))?;
        let path = path.join("editor.cfg");
        if path.exists() {
            let contents =
                std::fs::read_to_string(path).map_err(|err| crate::error::Error::IO(err, None))?;
            serde_json::from_str::<Settings>(contents.as_str())
                .map_err(|err| crate::error::Error::SerdeJsonError(err))
        } else {
            let default_settings = Settings::default();
            let contents = serde_json::to_string_pretty(&default_settings)
                .map_err(|err| crate::error::Error::SerdeJsonError(err))?;
            std::fs::write(path, contents).map_err(|err| crate::error::Error::IO(err, None))?;
            Ok(default_settings.clone())
        }
    }

    pub fn new_main_level(&self) -> Option<crate::content::level::Level> {
        let mut level: Option<crate::content::level::Level> = None;
        (|| {
            let mut resource_manager = ResourceManager::default();
            let Some(url) = Self::find_first_level(&mut resource_manager) else {
                return;
            };
            let Ok(_level) = resource_manager.get_level(&url) else {
                return;
            };
            log::trace!("Load level: {}", _level.url.to_string());
            level = Some(_level);
        })();
        level
    }

    fn collect_content_files() -> HashMap<url::Url, EContentFileType> {
        let resource_manager = ResourceManager::default();
        let mut files: HashMap<url::Url, EContentFileType> = HashMap::new();
        let Ok(resource_map) = resource_manager.get_resource_map() else {
            return files;
        };
        for (url, v) in resource_map.iter() {
            match v.resource_type {
                EResourceType::Content(content_ty) => match content_ty {
                    EContentType::StaticMesh => {
                        match resource_manager
                            .get_resource::<crate::content::static_mesh::StaticMesh>(
                                url,
                                Some(EResourceType::Content(EContentType::StaticMesh)),
                            ) {
                            Ok(static_mesh) => {
                                files.insert(
                                    url.clone(),
                                    EContentFileType::StaticMesh(SingleThreadMut::new(static_mesh)),
                                );
                            }
                            Err(err) => {
                                log::warn!("{err}");
                            }
                        }
                    }
                    EContentType::SkeletonMesh => {}
                    EContentType::SkeletonAnimation => {}
                    EContentType::Skeleton => {
                        match resource_manager.get_resource::<crate::content::skeleton::Skeleton>(
                            url,
                            Some(EResourceType::Content(EContentType::Skeleton)),
                        ) {
                            Ok(content_skeleton) => {
                                files.insert(
                                    url.clone(),
                                    EContentFileType::Skeleton(SingleThreadMut::new(
                                        content_skeleton,
                                    )),
                                );
                            }
                            Err(err) => {
                                log::warn!("{err}");
                            }
                        }
                    }
                    EContentType::Texture => {}
                    EContentType::Level => {}
                    EContentType::Material => {
                        match resource_manager
                            .get_resource::<crate::content::material::Material>(url, None)
                        {
                            Ok(f) => {
                                files.insert(
                                    url.clone(),
                                    EContentFileType::Material(SingleThreadMut::new(f)),
                                );
                            }
                            Err(err) => {
                                log::warn!("{}", err);
                            }
                        }
                    }
                    EContentType::IBL => {
                        match resource_manager.get_resource::<crate::content::ibl::IBL>(
                            url,
                            Some(EResourceType::Content(EContentType::IBL)),
                        ) {
                            Ok(ibl) => {
                                files.insert(
                                    url.clone(),
                                    EContentFileType::IBL(SingleThreadMut::new(ibl)),
                                );
                            }
                            Err(err) => {
                                log::warn!("{err}");
                            }
                        }
                    }
                    EContentType::MediaSource => todo!(),
                    EContentType::ParticleSystem => {
                        match resource_manager
                            .get_resource::<crate::content::particle_system::ParticleSystem>(
                                url,
                                Some(EResourceType::Content(EContentType::ParticleSystem)),
                            ) {
                            Ok(particle_system) => {
                                files.insert(
                                    url.clone(),
                                    EContentFileType::ParticleSystem(SingleThreadMut::new(
                                        particle_system,
                                    )),
                                );
                            }
                            Err(err) => {
                                log::warn!("{err}");
                            }
                        }
                    }
                    EContentType::Sound => match resource_manager
                        .get_resource::<crate::content::sound::Sound>(
                            url,
                            Some(EResourceType::Content(EContentType::Sound)),
                        ) {
                        Ok(sound) => {
                            files.insert(
                                url.clone(),
                                EContentFileType::Sound(SingleThreadMut::new(sound)),
                            );
                        }
                        Err(err) => {
                            log::warn!("{err}");
                        }
                    },
                    EContentType::Curve => todo!(),
                },
                _ => {}
            }
        }
        files
    }

    pub fn init_resources(&mut self) {
        let Ok(resource_map) = self.resource_manager.get_resource_map() else {
            return;
        };

        for (url, resource_info) in resource_map {
            match resource_info.resource_type {
                rs_artifact::resource_type::EResourceType::SkinMesh => {
                    if let Ok(skin_mesh) = self
                        .resource_manager
                        .get_resource::<rs_artifact::skin_mesh::SkinMesh>(
                            &url,
                            Some(resource_info.resource_type),
                        )
                    {
                        self.resource_manager
                            .add_skin_mesh(url.clone(), Arc::new(skin_mesh));
                    }
                }
                rs_artifact::resource_type::EResourceType::StaticMesh => {
                    if let Ok(static_mesh) = self
                        .resource_manager
                        .get_resource::<rs_artifact::static_mesh::StaticMesh>(
                        &url,
                        Some(resource_info.resource_type),
                    ) {
                        self.resource_manager
                            .add_static_mesh(url.clone(), Arc::new(static_mesh));
                    }
                }
                rs_artifact::resource_type::EResourceType::SkeletonAnimation => {
                    if let Ok(skeleton_animation) =
                        self.resource_manager
                            .get_resource::<rs_artifact::skeleton_animation::SkeletonAnimation>(
                                &url,
                                Some(resource_info.resource_type),
                            )
                    {
                        self.resource_manager
                            .add_skeleton_animation(url.clone(), Arc::new(skeleton_animation));
                    }
                }
                rs_artifact::resource_type::EResourceType::Skeleton => {
                    if let Ok(skeleton) = self
                        .resource_manager
                        .get_resource::<rs_artifact::skeleton::Skeleton>(
                            &url,
                            Some(resource_info.resource_type),
                        )
                    {
                        self.resource_manager
                            .add_skeleton(url.clone(), Arc::new(skeleton));
                    }
                }
                rs_artifact::resource_type::EResourceType::IBLBaking => {
                    if let Ok(ibl_baking) = self
                        .resource_manager
                        .get_resource::<rs_artifact::ibl_baking::IBLBaking>(
                            &url,
                            Some(resource_info.resource_type),
                        )
                    {
                        self.upload_prebake_ibl(ibl_baking.url.clone(), ibl_baking);
                    }
                }
                rs_artifact::resource_type::EResourceType::Material => {
                    if let Ok(material) = self
                        .resource_manager
                        .get_resource::<rs_artifact::material::Material>(
                            &url,
                            Some(resource_info.resource_type),
                        )
                    {
                        let material_content = self.content_files.values().find_map(|x| match x {
                            EContentFileType::Material(material_content) => {
                                if material_content.borrow().asset_url == material.url {
                                    Some(material_content.clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        });

                        if let Some(material_content) = material_content {
                            let pipeline_handle = self.create_material(material.code);
                            let mut material_content = material_content.borrow_mut();
                            material_content.set_pipeline_handle(pipeline_handle);
                            material_content.set_material_info(material.material_info);
                        }
                    }
                }
                rs_artifact::resource_type::EResourceType::Sound => {
                    if let Ok(sound) = self
                        .resource_manager
                        .get_resource::<rs_artifact::sound::Sound>(
                            &url,
                            Some(resource_info.resource_type),
                        )
                    {
                        let url = sound.url.clone();
                        self.resource_manager.add_sound(url, Arc::new(sound));
                    }
                }
                rs_artifact::resource_type::EResourceType::Content(content_type) => {
                    match content_type {
                        EContentType::Texture => {
                            let result: crate::error::Result<()> = (|| {
                                let texture = self
                                    .resource_manager
                                    .get_resource::<crate::content::texture::TextureFile>(
                                    &url,
                                    Some(resource_info.resource_type),
                                )?;

                                let image_reference = texture.image_reference.ok_or(
                                    crate::error::Error::NullReference(Some(
                                        "No image reference".to_string(),
                                    )),
                                )?;
                                log::trace!("Image reference: {}", image_reference.to_string());
                                let image = self
                                    .resource_manager
                                    .get_resource::<rs_artifact::image::Image>(
                                        &image_reference,
                                        Some(EResourceType::Image),
                                    )?;

                                let dyn_image = image::load_from_memory(&image.data)
                                    .map_err(|err| crate::error::Error::ImageError(err, None))?;
                                let rgba_image = match dyn_image.as_rgba8() {
                                    Some(_) => dyn_image.as_rgba8().unwrap().clone(),
                                    None => dyn_image.to_rgba8(),
                                };
                                log::trace!("{:?}", image.image_format);
                                self.create_texture_from_image(&url, &rgba_image)?;
                                Ok(())
                            })();
                            log::trace!("Laod texture: {}, {:?}", url.to_string(), result);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn find_first_level(resource_manager: &mut ResourceManager) -> Option<url::Url> {
        let Ok(resource_map) = resource_manager.get_resource_map() else {
            return None;
        };
        for (_, v) in resource_map {
            match v.resource_type {
                EResourceType::Content(content_type) => match content_type {
                    EContentType::Level => {
                        return Some(v.url);
                    }
                    _ => {
                        continue;
                    }
                },
                _ => {
                    continue;
                }
            }
        }
        return None;
    }

    pub fn get_resource_map(&self) -> Result<HashMap<url::Url, ResourceInfo>> {
        self.resource_manager.get_resource_map()
    }

    pub fn window_redraw_requested_begin(&mut self, window_id: isize) {
        self.render_thread_mode.recv_output();
        self.render_thread_mode
            .send_command(RenderCommand::WindowRedrawRequestedBegin(window_id));
    }

    pub fn window_redraw_requested_end(&mut self, window_id: isize) {
        self.render_thread_mode
            .send_command(RenderCommand::WindowRedrawRequestedEnd(window_id));
    }

    pub fn draw_gui(&mut self, gui_render_output: EGUIRenderOutput) {
        self.render_thread_mode
            .send_command(RenderCommand::UiOutput(gui_render_output));
    }

    pub fn present_player_viewport(&mut self, player_viewport: &mut PlayerViewport) {
        let command = RenderCommand::UpdateBuffer(UpdateBuffer {
            handle: *player_viewport.global_constants_handle,
            data: rs_foundation::cast_to_raw_buffer(&vec![player_viewport.global_constants])
                .to_vec(),
        });
        self.render_thread_mode.send_command(command);
        let mut draw_objects: Vec<_> = player_viewport.debug_draw_objects.drain(..).collect();
        draw_objects.append(&mut player_viewport.draw_objects.drain(..).collect());
        draw_objects.append(&mut player_viewport.particle_draw_objects.drain(..).collect());
        // let mut draw_objects: Vec<_> = player_viewport.draw_objects.drain(..).collect();
        if let Some(grid_draw_object) = player_viewport.get_grid_draw_object() {
            draw_objects.push(grid_draw_object.clone());
        }
        let virtual_texture_pass = player_viewport.virtual_pass_handle.clone().map(|x| x.key());
        if let Some(key) = virtual_texture_pass {
            self.render_thread_mode
                .send_command(RenderCommand::ClearVirtualTexturePass(key));
        }

        if let Some(shadow_depth_texture_handle) =
            player_viewport.shadow_depth_texture_handle.clone()
        {
            self.render_thread_mode
                .send_command(RenderCommand::ClearDepthTexture(ClearDepthTexture {
                    handle: *shadow_depth_texture_handle,
                }));
        }

        let virtual_texture_pass = player_viewport.virtual_pass_handle.clone().map(|x| x.key());
        self.render_thread_mode
            .send_command(RenderCommand::Present(PresentInfo {
                render_target_type: *player_viewport.get_render_target_type(),
                draw_objects,
                virtual_texture_pass,
                scene_viewport: player_viewport.scene_viewport.clone(),
            }));

        let pending_destroy_textures = ResourceManager::default().get_pending_destroy_textures();
        if !pending_destroy_textures.is_empty() {
            let pending_destroy_textures = pending_destroy_textures.iter().map(|x| **x).collect();
            self.render_thread_mode
                .send_command(RenderCommand::DestroyTextures(pending_destroy_textures));
        }
    }

    pub fn resize(&mut self, window_id: isize, surface_width: u32, surface_height: u32) {
        self.render_thread_mode.send_command(RenderCommand::Resize(
            rs_render::command::ResizeInfo {
                width: surface_width,
                height: surface_height,
                window_id,
            },
        ));
    }

    pub fn remove_window(&mut self, window_id: isize) {
        self.render_thread_mode
            .send_command(RenderCommand::RemoveWindow(window_id));
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
        self.render_thread_mode.set_new_window(
            window_id,
            window,
            surface_width,
            surface_height,
            scale_factor,
        )
    }

    fn next_draw_object_id(&mut self) -> u32 {
        let id = self.draw_object_id;
        self.draw_object_id += 1;
        id
    }

    fn convert_vertex(
        vertexes: &[rs_artifact::mesh_vertex::MeshVertex],
    ) -> (
        Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex0>,
        Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex1>,
    ) {
        let mut vertexes0: Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex0> =
            Vec::with_capacity(vertexes.len());
        let mut vertexes1: Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex1> =
            Vec::with_capacity(vertexes.len());

        for vertex in vertexes {
            vertexes0.push(rs_render::vertex_data_type::mesh_vertex::MeshVertex0 {
                position: vertex.position,
                tex_coord: vertex.tex_coord,
            });
            vertexes1.push(rs_render::vertex_data_type::mesh_vertex::MeshVertex1 {
                vertex_color: vertex.vertex_color,
                normal: vertex.normal,
                tangent: vertex.tangent,
                bitangent: vertex.bitangent,
            });
        }
        (vertexes0, vertexes1)
    }

    fn convert_vertex2(
        vertexes: &[rs_artifact::skin_mesh::SkinMeshVertex],
    ) -> (
        Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex0>,
        Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex1>,
        Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex2>,
    ) {
        let mut vertexes0: Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex0> =
            Vec::with_capacity(vertexes.len());
        let mut vertexes1: Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex1> =
            Vec::with_capacity(vertexes.len());
        let mut vertexes2: Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex2> =
            Vec::with_capacity(vertexes.len());

        for vertex in vertexes {
            vertexes0.push(rs_render::vertex_data_type::mesh_vertex::MeshVertex0 {
                position: vertex.position,
                tex_coord: vertex.tex_coord,
            });
            vertexes1.push(rs_render::vertex_data_type::mesh_vertex::MeshVertex1 {
                vertex_color: vertex.vertex_color,
                normal: vertex.normal,
                tangent: vertex.tangent,
                bitangent: vertex.bitangent,
            });
            vertexes2.push(rs_render::vertex_data_type::mesh_vertex::MeshVertex2 {
                bone_ids: vertex.bones.into(),
                bone_weights: vertex.weights.into(),
            });
        }
        (vertexes0, vertexes1, vertexes2)
    }

    pub fn create_draw_object_from_static_mesh(
        &mut self,
        vertexes: &[rs_artifact::mesh_vertex::MeshVertex],
        indexes: &[u32],
        name: Option<String>,
        global_constants_handle: crate::handle::BufferHandle,
    ) -> EDrawObjectType {
        let name = name.unwrap_or("".to_string());
        let (vertexes0, vertexes1) = Self::convert_vertex(vertexes);
        let id = self.next_draw_object_id();
        let index_buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("rs.IndexBuffer.{}", name.clone())),
            contents: rs_foundation::cast_to_raw_buffer(&indexes).to_vec(),
            usage: wgpu::BufferUsages::INDEX,
        };
        let create_buffer = CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.render_thread_mode.send_command(message);
        let vertex_buffers = vec![
            (
                format!("rs.{name}.MeshVertex0"),
                rs_foundation::cast_to_raw_buffer(&vertexes0),
            ),
            (
                format!("rs.{name}.MeshVertex1"),
                rs_foundation::cast_to_raw_buffer(&vertexes1),
            ),
        ];
        let mut vertex_buffer_handles: Vec<crate::handle::BufferHandle> =
            Vec::with_capacity(vertex_buffers.len());
        for (name, vertex_buffer) in vertex_buffers {
            let vertex_buffer_handle = self.resource_manager.next_buffer();
            let buffer_create_info = BufferCreateInfo {
                label: Some(format!("rs.{}.VertexBuffer", name)),
                contents: vertex_buffer.to_vec(),
                usage: wgpu::BufferUsages::VERTEX,
            };
            let create_buffer = CreateBuffer {
                handle: *vertex_buffer_handle,
                buffer_create_info,
            };
            let message = RenderCommand::CreateBuffer(create_buffer);
            self.render_thread_mode.send_command(message);
            vertex_buffer_handles.push(vertex_buffer_handle);
        }

        let constants_buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("rs.{}.Constants", name.clone())),
            contents: rs_foundation::cast_any_as_u8_slice(
                &rs_render::render_pipeline::shading::Constants::default(),
            )
            .to_vec(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
        };
        let create_buffer = CreateBuffer {
            handle: *constants_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.render_thread_mode.send_command(message);
        let global_sampler_handle = ResourceManager::default()
            .get_builtin_resources()
            .global_sampler_handle
            .clone();
        let object = StaticMeshDrawObject {
            id,
            vertex_buffers: vertex_buffer_handles,
            vertex_count: vertexes0.len() as u32,
            index_buffer: Some(index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            constants: Default::default(),
            diffuse_texture_url: Default::default(),
            specular_texture_url: Default::default(),
            constants_buffer_handle: constants_buffer_handle.clone(),
            window_id: self.main_window_id,
            global_constants_resource: EBindingResource::Constants(*global_constants_handle),
            base_color_sampler_resource: EBindingResource::Sampler(*global_sampler_handle),
            physical_texture_resource: EBindingResource::Texture(
                self.virtual_pass_handle
                    .clone()
                    .map(|x| x.key())
                    .unwrap()
                    .physical_texture_handle,
            ),
            page_table_texture_resource: EBindingResource::Texture(
                self.virtual_pass_handle
                    .clone()
                    .map(|x| x.key())
                    .unwrap()
                    .page_table_texture_handle,
            ),
            diffuse_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_texture_handle(),
            ),
            specular_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_texture_handle(),
            ),
            constants_resource: EBindingResource::Constants(*constants_buffer_handle),
        };
        EDrawObjectType::Static(object)
    }

    pub fn create_draw_object_from_skin_mesh(
        &mut self,
        vertexes: &[rs_artifact::skin_mesh::SkinMeshVertex],
        indexes: &[u32],
        name: Option<String>,
        global_constants_handle: crate::handle::BufferHandle,
    ) -> EDrawObjectType {
        let name = name.unwrap_or("".to_string());
        let (vertexes0, vertexes1, vertexes2) = Self::convert_vertex2(vertexes);
        let id = self.next_draw_object_id();
        let index_buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("rs.IndexBuffer.{}", name.clone())),
            contents: rs_foundation::cast_to_raw_buffer(&indexes).to_vec(),
            usage: wgpu::BufferUsages::INDEX,
        };
        let create_buffer = CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.render_thread_mode.send_command(message);
        let vertex_buffers = vec![
            (
                format!("rs.{name}.MeshVertex0"),
                rs_foundation::cast_to_raw_buffer(&vertexes0),
            ),
            (
                format!("rs.{name}.MeshVertex1"),
                rs_foundation::cast_to_raw_buffer(&vertexes1),
            ),
            (
                format!("rs.{name}.MeshVertex2"),
                rs_foundation::cast_to_raw_buffer(&vertexes2),
            ),
        ];
        let mut vertex_buffer_handles: Vec<crate::handle::BufferHandle> =
            Vec::with_capacity(vertex_buffers.len());
        for (name, vertex_buffer) in vertex_buffers {
            let vertex_buffer_handle = self.resource_manager.next_buffer();
            let buffer_create_info = BufferCreateInfo {
                label: Some(format!("rs.{}.VertexBuffer", name)),
                contents: vertex_buffer.to_vec(),
                usage: wgpu::BufferUsages::VERTEX,
            };
            let create_buffer = CreateBuffer {
                handle: *vertex_buffer_handle,
                buffer_create_info,
            };
            let message = RenderCommand::CreateBuffer(create_buffer);
            self.render_thread_mode.send_command(message);
            vertex_buffer_handles.push(vertex_buffer_handle);
        }

        let constants_buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("rs.{}.Constants", name.clone())),
            contents: rs_foundation::cast_any_as_u8_slice(
                &rs_render::render_pipeline::skin_mesh_shading::Constants::default(),
            )
            .to_vec(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
        };
        let create_buffer = CreateBuffer {
            handle: *constants_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.render_thread_mode.send_command(message);
        let global_sampler_handle = ResourceManager::default()
            .get_builtin_resources()
            .global_sampler_handle
            .clone();

        let object = SkinMeshDrawObject {
            id,
            vertex_buffers: vertex_buffer_handles,
            vertex_count: vertexes0.len() as u32,
            index_buffer: Some(index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            constants: Default::default(),
            diffuse_texture_url: Default::default(),
            specular_texture_url: Default::default(),
            constants_buffer_handle: constants_buffer_handle.clone(),
            window_id: self.main_window_id,
            global_constants_resource: EBindingResource::Constants(*global_constants_handle),
            base_color_sampler_resource: EBindingResource::Sampler(*global_sampler_handle),
            physical_texture_resource: EBindingResource::Texture(
                self.virtual_pass_handle
                    .clone()
                    .map(|x| x.key())
                    .unwrap()
                    .physical_texture_handle,
            ),
            page_table_texture_resource: EBindingResource::Texture(
                self.virtual_pass_handle
                    .clone()
                    .map(|x| x.key())
                    .unwrap()
                    .page_table_texture_handle,
            ),
            diffuse_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_texture_handle(),
            ),
            specular_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_texture_handle(),
            ),
            constants_resource: EBindingResource::Constants(*constants_buffer_handle),
        };
        EDrawObjectType::Skin(object)
    }

    pub fn create_material_draw_object_from_skin_mesh(
        &mut self,
        vertexes: &[rs_artifact::skin_mesh::SkinMeshVertex],
        indexes: &[u32],
        name: Option<String>,
        material: Rc<RefCell<crate::content::material::Material>>,
        global_constants_handle: crate::handle::BufferHandle,
    ) -> EDrawObjectType {
        let name = name.unwrap_or("".to_string());
        let (vertexes0, vertexes1, vertexes2) = Self::convert_vertex2(vertexes);
        let id = self.next_draw_object_id();
        let index_buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("rs.IndexBuffer.{}", name.clone())),
            contents: rs_foundation::cast_to_raw_buffer(&indexes).to_vec(),
            usage: wgpu::BufferUsages::INDEX,
        };
        let create_buffer = CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.render_thread_mode.send_command(message);
        let vertex_buffers = vec![
            (
                format!("rs.{name}.MeshVertex0"),
                rs_foundation::cast_to_raw_buffer(&vertexes0),
            ),
            (
                format!("rs.{name}.MeshVertex1"),
                rs_foundation::cast_to_raw_buffer(&vertexes1),
            ),
            (
                format!("rs.{name}.MeshVertex2"),
                rs_foundation::cast_to_raw_buffer(&vertexes2),
            ),
        ];
        let mut vertex_buffer_handles: Vec<crate::handle::BufferHandle> =
            Vec::with_capacity(vertex_buffers.len());
        for (name, vertex_buffer) in vertex_buffers {
            let vertex_buffer_handle = self.resource_manager.next_buffer();
            let buffer_create_info = BufferCreateInfo {
                label: Some(format!("rs.{}.VertexBuffer", name)),
                contents: vertex_buffer.to_vec(),
                usage: wgpu::BufferUsages::VERTEX,
            };
            let create_buffer = CreateBuffer {
                handle: *vertex_buffer_handle,
                buffer_create_info,
            };
            let message = RenderCommand::CreateBuffer(create_buffer);
            self.render_thread_mode.send_command(message);
            vertex_buffer_handles.push(vertex_buffer_handle);
        }

        let mut fn_create_buffer = |label: String, contents: Vec<u8>| {
            let constants_buffer_handle = self.resource_manager.next_buffer();
            let buffer_create_info = BufferCreateInfo {
                label: Some(label),
                contents,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
            };
            let create_buffer = CreateBuffer {
                handle: *constants_buffer_handle,
                buffer_create_info,
            };
            let message = RenderCommand::CreateBuffer(create_buffer);
            self.render_thread_mode.send_command(message);
            constants_buffer_handle
        };
        let constants_buffer_handle = fn_create_buffer(
            format!("rs.{}.Constants", name.clone()),
            rs_foundation::cast_any_as_u8_slice(&rs_render::constants::Constants::default())
                .to_vec(),
        );
        let skin_constants_buffer_handle = fn_create_buffer(
            format!("rs.{}.SkinConstants", name.clone()),
            rs_foundation::cast_any_as_u8_slice(&rs_render::constants::SkinConstants::default())
                .to_vec(),
        );
        let virtual_texture_constants_buffer_handle = fn_create_buffer(
            format!("rs.{}.VirtualTextureConstants", name.clone()),
            rs_foundation::cast_any_as_u8_slice(
                &rs_render::constants::VirtualTextureConstants::default(),
            )
            .to_vec(),
        );
        let global_sampler_handle = ResourceManager::default()
            .get_builtin_resources()
            .global_sampler_handle
            .clone();

        let object = MaterialDrawObject {
            id,
            vertex_buffers: vertex_buffer_handles,
            vertex_count: vertexes0.len() as u32,
            index_buffer: Some(index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            global_constants_resource: EBindingResource::Constants(*global_constants_handle),
            base_color_sampler_resource: EBindingResource::Sampler(*global_sampler_handle),
            physical_texture_resource: EBindingResource::Texture(
                self.virtual_pass_handle
                    .clone()
                    .map(|x| x.key())
                    .unwrap()
                    .physical_texture_handle,
            ),
            page_table_texture_resource: EBindingResource::Texture(
                self.virtual_pass_handle
                    .clone()
                    .map(|x| x.key())
                    .unwrap()
                    .page_table_texture_handle,
            ),
            material,
            constants_buffer_handle: constants_buffer_handle.clone(),
            skin_constants_buffer_handle: skin_constants_buffer_handle.clone(),
            virtual_texture_constants_buffer_handle: virtual_texture_constants_buffer_handle
                .clone(),
            constants: Default::default(),
            skin_constants: Default::default(),
            virtual_texture_constants: Default::default(),
            window_id: self.main_window_id,
            brdflut_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_ibl_textures().brdflut,
            ),
            pre_filter_cube_map_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_ibl_textures().pre_filter_cube_map,
            ),
            irradiance_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_ibl_textures().irradiance,
            ),
            constants_resource: EBindingResource::Constants(*constants_buffer_handle),
            skin_constants_resource: EBindingResource::Constants(*skin_constants_buffer_handle),
            virtual_texture_constants_resource: EBindingResource::Constants(
                *virtual_texture_constants_buffer_handle,
            ),
            user_textures_resources: vec![],
            shadow_map_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_depth_texture_handle(),
            ),
        };
        EDrawObjectType::SkinMaterial(object)
    }

    pub fn create_gpu_buffer<T: Sized>(
        &mut self,
        contents: &[T],
        usage: wgpu::BufferUsages,
        label: Option<String>,
    ) -> crate::handle::BufferHandle {
        let buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label,
            contents: rs_foundation::cast_to_raw_buffer(&contents).to_vec(),
            usage,
        };
        let create_buffer = CreateBuffer {
            handle: *buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.render_thread_mode.send_command(message);
        buffer_handle
    }

    pub fn create_constants_buffer<T: Sized>(
        &mut self,
        contents: &[T],
        label: Option<String>,
    ) -> crate::handle::BufferHandle {
        self.create_gpu_buffer(
            contents,
            wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::MAP_WRITE
                | wgpu::BufferUsages::COPY_DST,
            label,
        )
    }

    pub fn create_vertex_buffer<T: Sized>(
        &mut self,
        contents: &[T],
        label: Option<String>,
    ) -> crate::handle::BufferHandle {
        self.create_gpu_buffer(contents, wgpu::BufferUsages::VERTEX, label)
    }

    pub fn create_material_draw_object_from_static_mesh(
        &mut self,
        vertexes: &[rs_artifact::mesh_vertex::MeshVertex],
        indexes: &[u32],
        name: Option<String>,
        material: Rc<RefCell<crate::content::material::Material>>,
        global_constants_handle: crate::handle::BufferHandle,
    ) -> EDrawObjectType {
        let name = name.unwrap_or("".to_string());
        let (vertexes0, vertexes1) = Self::convert_vertex(vertexes);
        let id = self.next_draw_object_id();
        let index_buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("rs.IndexBuffer.{}", name.clone())),
            contents: rs_foundation::cast_to_raw_buffer(&indexes).to_vec(),
            usage: wgpu::BufferUsages::INDEX,
        };
        let create_buffer = CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.render_thread_mode.send_command(message);
        let vertex_buffers = vec![
            (
                format!("rs.{name}.MeshVertex0"),
                rs_foundation::cast_to_raw_buffer(&vertexes0),
            ),
            (
                format!("rs.{name}.MeshVertex1"),
                rs_foundation::cast_to_raw_buffer(&vertexes1),
            ),
        ];
        let mut vertex_buffer_handles: Vec<crate::handle::BufferHandle> =
            Vec::with_capacity(vertex_buffers.len());
        for (name, vertex_buffer) in vertex_buffers {
            let vertex_buffer_handle = self.resource_manager.next_buffer();
            let buffer_create_info = BufferCreateInfo {
                label: Some(format!("rs.{}.VertexBuffer", name)),
                contents: vertex_buffer.to_vec(),
                usage: wgpu::BufferUsages::VERTEX,
            };
            let create_buffer = CreateBuffer {
                handle: *vertex_buffer_handle,
                buffer_create_info,
            };
            let message = RenderCommand::CreateBuffer(create_buffer);
            self.render_thread_mode.send_command(message);
            vertex_buffer_handles.push(vertex_buffer_handle);
        }

        let mut fn_create_buffer = |label: String, contents: Vec<u8>| {
            let constants_buffer_handle = self.resource_manager.next_buffer();
            let buffer_create_info = BufferCreateInfo {
                label: Some(label),
                contents,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
            };
            let create_buffer = CreateBuffer {
                handle: *constants_buffer_handle,
                buffer_create_info,
            };
            let message = RenderCommand::CreateBuffer(create_buffer);
            self.render_thread_mode.send_command(message);
            constants_buffer_handle
        };
        let constants_buffer_handle = fn_create_buffer(
            format!("rs.{}.Constants", name.clone()),
            rs_foundation::cast_any_as_u8_slice(&rs_render::constants::Constants::default())
                .to_vec(),
        );

        let virtual_texture_constants_buffer_handle = fn_create_buffer(
            format!("rs.{}.VirtualTextureConstants", name.clone()),
            rs_foundation::cast_any_as_u8_slice(
                &rs_render::constants::VirtualTextureConstants::default(),
            )
            .to_vec(),
        );
        let global_sampler_handle = ResourceManager::default()
            .get_builtin_resources()
            .global_sampler_handle
            .clone();

        let object = StaticMeshMaterialDrawObject {
            id,
            vertex_buffers: vertex_buffer_handles,
            vertex_count: vertexes0.len() as u32,
            index_buffer: Some(index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            global_constants_resource: EBindingResource::Constants(*global_constants_handle),
            base_color_sampler_resource: EBindingResource::Sampler(*global_sampler_handle),
            physical_texture_resource: EBindingResource::Texture(
                self.virtual_pass_handle
                    .clone()
                    .map(|x| x.key())
                    .unwrap()
                    .physical_texture_handle,
            ),
            page_table_texture_resource: EBindingResource::Texture(
                self.virtual_pass_handle
                    .clone()
                    .map(|x| x.key())
                    .unwrap()
                    .page_table_texture_handle,
            ),
            material,
            constants_buffer_handle: constants_buffer_handle.clone(),
            virtual_texture_constants_buffer_handle: virtual_texture_constants_buffer_handle
                .clone(),
            constants: Default::default(),
            virtual_texture_constants: Default::default(),
            window_id: self.main_window_id,
            brdflut_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_ibl_textures().brdflut,
            ),
            pre_filter_cube_map_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_ibl_textures().pre_filter_cube_map,
            ),
            irradiance_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_ibl_textures().irradiance,
            ),
            constants_resource: EBindingResource::Constants(*constants_buffer_handle),
            virtual_texture_constants_resource: EBindingResource::Constants(
                *virtual_texture_constants_buffer_handle,
            ),
            user_textures_resources: vec![],
            shadow_map_texture_resource: EBindingResource::Texture(
                *self.default_textures.get_depth_texture_handle(),
            ),
        };
        EDrawObjectType::StaticMeshMaterial(object)
    }

    pub fn get_mut_resource_manager(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }

    pub fn create_texture(
        &mut self,
        url: &url::Url,
        info: TextureDescriptorCreateInfo,
    ) -> crate::handle::TextureHandle {
        let handle = self.resource_manager.next_texture(url.clone());
        let create_texture = CreateTexture {
            handle: *handle,
            texture_descriptor_create_info: info,
            init_data: None,
        };
        let render_command = RenderCommand::CreateTexture(create_texture);
        self.render_thread_mode.send_command(render_command);
        handle
    }

    pub fn create_texture_from_image(
        &mut self,
        url: &url::Url,
        image: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    ) -> Result<crate::handle::TextureHandle> {
        let handle = self.resource_manager.next_texture(url.clone());
        let create_texture = CreateTexture {
            handle: *handle,
            texture_descriptor_create_info: TextureDescriptorCreateInfo::d2(
                Some(String::from(format!("{:?}", url.as_str()))),
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
        let render_command = RenderCommand::CreateTexture(create_texture);
        self.render_thread_mode.send_command(render_command);
        Ok(handle)
    }

    pub fn create_texture_from_path(
        &mut self,
        path: &Path,
        url: &url::Url,
    ) -> Result<crate::handle::TextureHandle> {
        let dynamic_image =
            image::open(path).map_err(|err| crate::error::Error::ImageError(err, None))?;
        let image = match dynamic_image {
            image::DynamicImage::ImageRgba8(image) => image,
            x => x.to_rgba8(),
        };
        self.create_texture_from_image(url, &image)
    }

    pub fn create_virtual_texture_source(
        &mut self,
        url: url::Url,
        source: Box<dyn TVirtualTextureSource>,
    ) {
        let mut virtual_texture_source_infos = self.virtual_texture_source_infos.borrow_mut();
        let ref_source = MultipleThreadMut::new(source);
        virtual_texture_source_infos.insert(url.clone(), ref_source.clone());
        let handle = self.resource_manager.next_virtual_texture(url.clone());
        let command = CreateVirtualTexture {
            handle: *handle,
            source: ref_source.clone(),
        };
        let render_command = RenderCommand::CreateVirtualTextureSource(command);
        self.render_thread_mode.send_command(render_command);
    }

    pub fn send_render_task(
        &mut self,
        task: impl FnMut(&mut rs_render::renderer::Renderer) + Send + 'static,
    ) {
        self.render_thread_mode
            .send_command(RenderCommand::create_task(task));
    }

    pub fn ibl_bake<P: AsRef<Path>>(
        &mut self,
        path: P,
        url: url::Url,
        bake_info: BakeInfo,
        save_dir: Option<P>,
    ) {
        let ibl_textures = self.resource_manager.next_ibl_textures(url.clone());
        let render_command = RenderCommand::CreateIBLBake(CreateIBLBake {
            key: ibl_textures.to_key(),
            file_path: path.as_ref().to_path_buf(),
            bake_info,
            save_dir: save_dir.map_or(None, |x| Some(x.as_ref().to_path_buf())),
        });
        self.render_thread_mode.send_command(render_command);
    }

    pub fn upload_prebake_ibl(
        &mut self,
        url: url::Url,
        ibl_baking: rs_artifact::ibl_baking::IBLBaking,
    ) {
        let ibl_textures = self.resource_manager.next_ibl_textures(url.clone());
        let render_command = RenderCommand::UploadPrebakeIBL(UploadPrebakeIBL {
            key: ibl_textures.to_key(),
            brdf_data: ibl_baking.brdf_data,
            pre_filter_data: ibl_baking.pre_filter_data,
            irradiance_data: ibl_baking.irradiance_data,
        });
        self.render_thread_mode.send_command(render_command);
    }

    pub fn debug_capture_frame(&mut self) {
        #[cfg(feature = "renderdoc")]
        {
            let render_command = RenderCommand::CaptureFrame;
            self.render_thread_mode.send_command(render_command);
        }
    }

    pub fn set_settings(&mut self, settings: Settings) {
        let render_command = RenderCommand::Settings(settings.render_setting.clone());
        self.render_thread_mode.send_command(render_command);
        self.settings = settings;
    }

    pub fn tick(&mut self) {
        let now = std::time::Instant::now();
        self.game_time_sec += (now - self.game_time).as_secs_f32();
        self.game_time = now;
    }

    pub fn get_game_time(&self) -> f32 {
        self.game_time_sec
    }

    pub fn set_view_mode(&mut self, view_mode: EViewModeType) {
        self.render_thread_mode
            .send_command(RenderCommand::ChangeViewMode(view_mode));
    }

    pub fn create_buffer(
        &mut self,
        buffer: &[u8],
        usage: wgpu::BufferUsages,
        name: Option<&str>,
    ) -> crate::handle::BufferHandle {
        let handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("{}", name.clone().unwrap_or(""))),
            contents: buffer.to_vec(),
            usage,
        };
        let create_buffer = CreateBuffer {
            handle: *handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        self.render_thread_mode.send_command(message);
        handle.clone()
    }

    pub fn update_buffer(&mut self, handle: crate::handle::BufferHandle, buffer: &[u8]) {
        let update_buffer = rs_render::command::UpdateBuffer {
            handle: *handle,
            data: buffer.to_vec(),
        };
        let message = RenderCommand::UpdateBuffer(update_buffer);
        self.render_thread_mode.send_command(message);
    }

    pub fn get_console_cmd_mut(&self, key: &str) -> Option<SingleThreadMutType<ConsoleCmd>> {
        self.console_cmds.borrow().get(key).cloned()
    }

    pub fn insert_console_cmd(&mut self, key: &str, c: ConsoleCmd) {
        self.console_cmds
            .borrow_mut()
            .insert(key.to_string(), SingleThreadMut::new(c));
    }

    pub fn get_console_cmds(
        &self,
    ) -> SingleThreadMutType<HashMap<String, SingleThreadMutType<ConsoleCmd>>> {
        self.console_cmds.clone()
    }

    #[cfg(feature = "editor")]
    pub fn create_grid_draw_object(
        &mut self,
        // id: u32,
        global_constants_handle: crate::handle::BufferHandle,
    ) -> rs_render::command::DrawObject {
        let resource_manager = ResourceManager::default();
        let id = self.next_draw_object_id();
        Self::internal_create_grid_draw_object(
            id,
            resource_manager,
            &mut self.render_thread_mode,
            global_constants_handle,
        )
    }

    #[cfg(feature = "editor")]
    fn internal_create_grid_draw_object(
        // window_id:isize,
        id: u32,
        resource_manager: ResourceManager,
        render_thread_mode: &mut ERenderThreadMode,
        global_constants_handle: crate::handle::BufferHandle,
    ) -> rs_render::command::DrawObject {
        let grid_data = rs_core_minimal::primitive_data::PrimitiveData::quad();
        let name = "Grid";
        let index_buffer_handle = resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("rs.IndexBuffer.{}", name)),
            contents: rs_foundation::cast_to_raw_buffer(&grid_data.indices).to_vec(),
            usage: wgpu::BufferUsages::INDEX,
        };
        let create_buffer = CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        render_thread_mode.send_command(message);

        let mut vertexes0: Vec<rs_render::vertex_data_type::mesh_vertex::MeshVertex0> = vec![];

        for (position, tex_coord) in
            std::iter::zip(&grid_data.vertex_positions, &grid_data.vertex_tex_coords)
        {
            vertexes0.push(rs_render::vertex_data_type::mesh_vertex::MeshVertex0 {
                position: *position,
                tex_coord: *tex_coord,
            });
        }
        let vertex_buffers = vec![(
            format!("rs.{name}.MeshVertex0"),
            rs_foundation::cast_to_raw_buffer(&vertexes0),
        )];
        let mut vertex_buffer_handles: Vec<crate::handle::BufferHandle> =
            Vec::with_capacity(vertex_buffers.len());
        for (name, vertex_buffer) in vertex_buffers {
            let vertex_buffer_handle = resource_manager.next_buffer();
            let buffer_create_info = BufferCreateInfo {
                label: Some(format!("rs.{}.VertexBuffer", name)),
                contents: vertex_buffer.to_vec(),
                usage: wgpu::BufferUsages::VERTEX,
            };
            let create_buffer = CreateBuffer {
                handle: *vertex_buffer_handle,
                buffer_create_info,
            };
            let message = RenderCommand::CreateBuffer(create_buffer);
            render_thread_mode.send_command(message);
            vertex_buffer_handles.push(vertex_buffer_handle);
        }
        let draw_object = rs_render::command::DrawObject::new(
            id,
            vertex_buffer_handles.iter().map(|x| **x).collect(),
            vertexes0.len() as u32,
            rs_render::renderer::EPipelineType::Builtin(
                rs_render::renderer::EBuiltinPipelineType::Grid,
            ),
            Some(*index_buffer_handle),
            Some(grid_data.indices.len() as u32),
            vec![vec![EBindingResource::Constants(*global_constants_handle)]],
        );
        draw_object
    }

    pub fn create_material(
        &mut self,
        shader_code: HashMap<MaterialOptions, String>,
    ) -> crate::handle::MaterialRenderPipelineHandle {
        let shader_handle = self.resource_manager.next_material_render_pipeline();
        self.render_thread_mode
            .send_command(RenderCommand::CreateMaterialRenderPipeline(
                CreateMaterialRenderPipeline {
                    handle: *shader_handle,
                    shader_code,
                },
            ));
        shader_handle.clone()
    }

    pub fn sdf2d(&mut self, image: image::RgbaImage) {
        self.render_thread_mode
            .send_command(RenderCommand::create_task(move |renderer| {
                let device = renderer.get_device();
                let queue = renderer.get_queue();
                let mut generator =
                    sdf2d_generator::Sdf2dGenerator::new(device, renderer.get_shader_library());
                generator.run(device, queue, &image, 0, 0.5);
            }));
    }

    pub fn send_render_command(&mut self, command: RenderCommand) {
        self.render_thread_mode.send_command(command);
    }

    pub fn create_ui_texture(
        &mut self,
        handle: EGUITextureHandle,
        referencing_texture_handle: TextureHandle,
    ) {
        self.render_thread_mode
            .send_command(RenderCommand::CreateUITexture(CreateUITexture {
                handle: *handle,
                referencing_texture_handle: *referencing_texture_handle,
            }));
    }

    pub fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    pub fn get_render_thread_mode_mut(&mut self) -> &mut ERenderThreadMode {
        &mut self.render_thread_mode
    }

    pub fn get_main_window_id(&self) -> isize {
        self.main_window_id
    }

    #[track_caller]
    pub fn log_trace(&self, message: &str) {
        log::trace!("{}", message);
    }

    pub fn get_logger_mut(&mut self) -> &mut Logger {
        &mut self.logger
    }

    pub fn get_virtual_texture_source_infos(
        &self,
    ) -> SingleThreadMutType<HashMap<url::Url, MultipleThreadMutType<Box<dyn TVirtualTextureSource>>>>
    {
        self.virtual_texture_source_infos.clone()
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn get_default_textures(&self) -> &DefaultTextures {
        &self.default_textures
    }

    #[cfg(not(target_os = "android"))]
    pub fn update_window_with_input_mode(
        window: &winit::window::Window,
        input_mode: crate::input_mode::EInputMode,
    ) {
        use winit::window::CursorGrabMode;
        match input_mode {
            crate::input_mode::EInputMode::Game => {
                window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                window.set_cursor_visible(false);
            }
            crate::input_mode::EInputMode::UI => {
                window.set_cursor_grab(CursorGrabMode::None).unwrap();
                window.set_cursor_visible(true);
            }
            crate::input_mode::EInputMode::GameUI => {
                window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                window.set_cursor_visible(true);
            }
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.logger.flush();
    }
}
