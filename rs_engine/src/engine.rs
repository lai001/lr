use crate::actor::Actor;
use crate::camera::Camera;
#[cfg(not(target_os = "android"))]
use crate::camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle};
use crate::console_cmd::ConsoleCmd;
use crate::content::content_file_type::EContentFileType;
use crate::default_textures::DefaultTextures;
use crate::directional_light::DirectionalLight;
use crate::drawable::{
    EDrawObjectType, MaterialDrawObject, SkinMeshDrawObject, StaticMeshDrawObject,
    StaticMeshMaterialDrawObject,
};
use crate::error::Result;
use crate::handle::{EGUITextureHandle, TextureHandle};
use crate::input_mode::EInputMode;
use crate::player_viewport::PlayerViewport;
use crate::render_thread_mode::ERenderThreadMode;
use crate::scene_node::EComponentType;
use crate::{build_built_in_resouce_url, BUILT_IN_RESOURCE};
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
    DrawObject, EBindingResource, InitTextureData, PresentInfo, RenderCommand, ShadowMapping,
    TextureDescriptorCreateInfo, UpdateBuffer, UploadPrebakeIBL, VirtualPassSet,
    VirtualTexturePassKey, VirtualTexturePassResize,
};
use rs_render::egui_render::EGUIRenderOutput;
use rs_render::global_uniform::{self, EDebugShadingType};
use rs_render::renderer::{EBuiltinPipelineType, EPipelineType, MaterialPipelineType, Renderer};
use rs_render::sdf2d_generator;
use rs_render::view_mode::EViewModeType;
use rs_render::virtual_texture_source::TVirtualTextureSource;
use rs_render_types::MaterialOptions;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

struct State {
    camera_movement_speed: f32,
    #[cfg(not(target_os = "android"))]
    camera_motion_speed: f32,
    #[cfg(not(target_os = "android"))]
    virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            camera_movement_speed: 0.01,
            #[cfg(not(target_os = "android"))]
            camera_motion_speed: 0.1,
            #[cfg(not(target_os = "android"))]
            virtual_key_code_states: Default::default(),
        }
    }
}

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
    level: Option<SingleThreadMutType<crate::content::level::Level>>,
    // draw_objects: Vec<DrawObject>,
    draw_object_id: u32,
    camera: Camera,
    state: State,
    settings: Settings,
    game_time: std::time::Instant,
    game_time_sec: f32,
    input_mode: EInputMode,
    global_constants: rs_render::global_uniform::Constants,
    global_constants_handle: crate::handle::BufferHandle,
    global_sampler_handle: crate::handle::SamplerHandle,
    virtual_texture_source_infos: SingleThreadMutType<
        HashMap<url::Url, MultipleThreadMutType<Box<dyn TVirtualTextureSource>>>,
    >,
    console_cmds: SingleThreadMutType<HashMap<String, SingleThreadMutType<ConsoleCmd>>>,
    grid_draw_object: Option<DrawObject>,
    content_files: HashMap<url::Url, EContentFileType>,
    main_window_id: isize,
    draw_objects: HashMap<isize, Vec<DrawObject>>,
    default_textures: DefaultTextures,
    virtual_pass_handle: Option<VirtualPassHandle>,
    shadow_depth_texture_handle: Option<TextureHandle>,
    player_viewports: Vec<SingleThreadMutType<PlayerViewport>>,
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

        let mut resource_manager = ResourceManager::default();
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
        let mut level: Option<SingleThreadMutType<crate::content::level::Level>> = None;
        (|| {
            let Some(url) = Self::find_first_level(&mut resource_manager) else {
                return;
            };
            let Ok(_level) = resource_manager.get_level(&url) else {
                return;
            };
            log::trace!("Load level: {}", _level.url.to_string());
            level = Some(SingleThreadMut::new(_level));
        })();

        #[cfg(feature = "editor")]
        let mut draw_object_id: u32 = 0;
        #[cfg(not(feature = "editor"))]
        let draw_object_id: u32 = 0;

        #[cfg(feature = "editor")]
        let grid_draw_object = (|| {
            draw_object_id += 1;
            Some(Self::internal_create_grid_draw_object(
                // window_id,
                draw_object_id,
                resource_manager.clone(),
                &mut render_thread_mode,
                global_constants_handle.clone(),
            ))
        })();
        #[cfg(not(feature = "editor"))]
        let grid_draw_object = None;

        #[cfg(feature = "editor")]
        let input_mode = EInputMode::UI;
        #[cfg(not(feature = "editor"))]
        let input_mode = EInputMode::Game;
        let default_textures = DefaultTextures::new(ResourceManager::default());
        default_textures.create(&mut render_thread_mode);

        let shadow_depth_texture_handle = resource_manager
            .next_texture(build_built_in_resouce_url("ShadowDepthTexture").unwrap());
        render_thread_mode.send_command(RenderCommand::CreateTexture(CreateTexture {
            handle: *shadow_depth_texture_handle,
            texture_descriptor_create_info: TextureDescriptorCreateInfo {
                label: Some(format!("ShadowDepthTexture")),
                size: wgpu::Extent3d {
                    width: 1024,
                    height: 1024,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: None,
            },
            init_data: None,
        }));
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
            level,
            draw_objects: HashMap::new(),
            camera,
            state: State::default(),
            settings: settings.clone(),
            draw_object_id,
            game_time: std::time::Instant::now(),
            game_time_sec: 0.0,
            input_mode,
            global_constants,
            global_constants_handle: global_constants_handle.clone(),
            global_sampler_handle: global_sampler_handle.clone(),
            virtual_texture_source_infos: virtual_texture_source_infos.clone(),
            console_cmds: SingleThreadMut::new(HashMap::new()),
            grid_draw_object,
            content_files: Self::collect_content_files(),
            main_window_id: window_id,
            default_textures,
            virtual_pass_handle,
            shadow_depth_texture_handle: Some(shadow_depth_texture_handle),
            player_viewports: vec![],
            _audio_device: Some(audio_device),
        };

        let mut player_viewport = PlayerViewport::new(
            window_id,
            surface_width,
            surface_height,
            global_sampler_handle,
            &mut engine,
            virtual_texture_source_infos.clone(),
            EInputMode::Game,
        );
        match &settings.render_setting.antialias_type {
            rs_core_minimal::settings::EAntialiasType::None => {}
            rs_core_minimal::settings::EAntialiasType::FXAA => {
                player_viewport.enable_fxaa(&mut engine);
            }
            rs_core_minimal::settings::EAntialiasType::MSAA => {
                player_viewport.enable_msaa(&mut engine);
            }
        }
        engine
            .player_viewports
            .push(SingleThreadMut::new(player_viewport));

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

    fn collect_content_files() -> HashMap<url::Url, EContentFileType> {
        let resource_manager = ResourceManager::default();
        let mut files: HashMap<url::Url, EContentFileType> = HashMap::new();
        if let Ok(resource_map) = resource_manager.get_resource_map() {
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
                                        EContentFileType::StaticMesh(SingleThreadMut::new(
                                            static_mesh,
                                        )),
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
                            match resource_manager
                                .get_resource::<crate::content::skeleton::Skeleton>(
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
                        EContentType::ParticleSystem => todo!(),
                        EContentType::Sound => todo!(),
                    },
                    _ => {}
                }
            }
        }
        files
    }

    pub fn init_level(&mut self) {
        if let Ok(resource_map) = self.resource_manager.get_resource_map() {
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
                        ) {
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
                            let material_content =
                                self.content_files.values().find_map(|x| match x {
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

                                    let dyn_image =
                                        image::load_from_memory(&image.data).map_err(|err| {
                                            crate::error::Error::ImageError(err, None)
                                        })?;
                                    let rgba_image = match dyn_image.as_rgba8() {
                                        Some(_) => dyn_image.as_rgba8().unwrap().clone(),
                                        None => dyn_image.to_rgba8(),
                                    };
                                    log::trace!("{:?}", image.image_format);
                                    self.create_texture_from_image(&url, &rgba_image)?;
                                    Ok(())
                                })(
                                );
                                log::trace!("Laod texture: {}, {:?}", url.to_string(), result);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        let actors: Vec<SingleThreadMutType<Actor>> = (|| {
            let Some(level) = self.level.as_mut().map(|x| x.borrow_mut()) else {
                log::warn!("{}", "No level found.");
                return vec![];
            };
            return level.actors.clone();
        })();
        for actor in actors {
            let mut actor = actor.borrow_mut();
            let root_scene_node = &mut actor.scene_node;
            let mut root_scene_node = root_scene_node.borrow_mut();
            match &mut root_scene_node.component {
                crate::scene_node::EComponentType::SceneComponent(_) => todo!(),
                crate::scene_node::EComponentType::StaticMeshComponent(static_mesh_component) => {
                    let mut static_mesh_component = static_mesh_component.borrow_mut();
                    let files: Vec<EContentFileType> =
                        self.content_files.values().map(|x| x.clone()).collect();
                    static_mesh_component.initialize(ResourceManager::default(), self, &files);
                }
                crate::scene_node::EComponentType::SkeletonMeshComponent(
                    skeleton_mesh_component,
                ) => {
                    let mut files: Vec<EContentFileType> = vec![];
                    if let Some(url) = skeleton_mesh_component.borrow().skeleton_url.as_ref() {
                        match self
                            .resource_manager
                            .get_resource::<crate::content::skeleton::Skeleton>(
                                url,
                                Some(EResourceType::Content(EContentType::Skeleton)),
                            ) {
                            Ok(content_skeleton) => {
                                files.push(EContentFileType::Skeleton(SingleThreadMut::new(
                                    content_skeleton,
                                )));
                            }
                            Err(err) => {
                                log::warn!("{err}");
                            }
                        }
                    }
                    if let Some(url) = skeleton_mesh_component.borrow().animation_url.as_ref() {
                        match self
                            .resource_manager
                            .get_resource::<crate::content::skeleton_animation::SkeletonAnimation>(
                            url,
                            Some(EResourceType::Content(EContentType::SkeletonAnimation)),
                        ) {
                            Ok(skeleton_animation) => {
                                files.push(EContentFileType::SkeletonAnimation(
                                    SingleThreadMut::new(skeleton_animation),
                                ));
                            }
                            Err(err) => {
                                log::warn!("{err}");
                            }
                        }
                    }

                    for url in &skeleton_mesh_component.borrow().skeleton_mesh_urls {
                        match self
                            .resource_manager
                            .get_resource::<crate::content::skeleton_mesh::SkeletonMesh>(
                                url,
                                Some(EResourceType::Content(EContentType::SkeletonMesh)),
                            ) {
                            Ok(skeleton_mesh) => {
                                files.push(EContentFileType::SkeletonMesh(SingleThreadMut::new(
                                    skeleton_mesh,
                                )));
                            }
                            Err(err) => {
                                log::warn!("{err}");
                            }
                        }
                    }
                    files.extend(
                        self.content_files
                            .values()
                            .cloned()
                            .collect::<Vec<EContentFileType>>(),
                    );
                    skeleton_mesh_component.borrow_mut().initialize(
                        ResourceManager::default(),
                        self,
                        &files,
                    );
                }
            }
        }
        if let Some(level) = self.level.clone().as_mut() {
            let mut level = level.borrow_mut();
            level.initialize(self);
            level.set_physics_simulate(true);
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

    pub fn recv_output_hook(&mut self) {
        // TODO
        self.render_thread_mode.recv_output();
    }

    pub fn redraw(&mut self, gui_render_output: EGUIRenderOutput) {
        self.render_thread_mode.recv_output();
        #[cfg(not(target_os = "android"))]
        for (virtual_key_code, element_state) in &self.state.virtual_key_code_states {
            DefaultCameraInputEventHandle::keyboard_input_handle(
                &mut self.camera,
                virtual_key_code,
                element_state,
                self.input_mode,
                self.state.camera_movement_speed,
            );
        }

        self.camera_did_update(
            self.camera.get_view_matrix(),
            self.camera.get_projection_matrix(),
            self.camera.get_world_location(),
        );

        let virtual_texture_setting = &self.settings.render_setting.virtual_texture_setting;
        self.global_constants.physical_texture_size =
            virtual_texture_setting.physical_texture_size as f32;
        self.global_constants.is_enable_virtual_texture = if virtual_texture_setting.is_enable {
            1
        } else {
            0
        };
        self.global_constants.tile_size = virtual_texture_setting.tile_size as f32;
        self.global_constants.scene_factor = virtual_texture_setting.feed_back_texture_div as f32;
        self.global_constants.feedback_bias = virtual_texture_setting.feedback_bias;
        self.update_global_constants();

        let actors: Vec<SingleThreadMutType<Actor>> = (|| {
            let Some(level) = self.level.as_mut().map(|x| x.borrow_mut()) else {
                return vec![];
            };
            return level.actors.clone();
        })();
        let time = self.get_game_time();
        let mut level = self.level.clone();
        if let Some(level) = level.clone() {
            level.borrow_mut().tick();
        }
        for actor in actors {
            match &mut actor.borrow_mut().scene_node.borrow_mut().component {
                EComponentType::SceneComponent(_) => todo!(),
                EComponentType::StaticMeshComponent(static_mesh_component) => {
                    let mut static_mesh_component = static_mesh_component.borrow_mut();
                    if let Some(level) = level.as_mut() {
                        let mut level = level.borrow_mut();
                        let rigid_body_set = level.get_rigid_body_set_mut();
                        static_mesh_component.update(time, self, rigid_body_set);
                    } else {
                        static_mesh_component.update(time, self, None);
                    }
                    for draw_object in static_mesh_component.get_draw_objects_mut() {
                        self.update_draw_object(draw_object);
                        self.draw2(draw_object);
                    }
                }
                EComponentType::SkeletonMeshComponent(skeleton_mesh_component) => {
                    let mut skeleton_mesh_component = skeleton_mesh_component.borrow_mut();
                    skeleton_mesh_component.update(self.get_game_time(), self);
                    for draw_object in skeleton_mesh_component.get_draw_objects() {
                        self.draw2(draw_object);
                    }
                }
            }
        }

        // for draw_object in &self.draw_objects {
        //     self.render_thread_mode
        //         .send_command(RenderCommand::DrawObject(draw_object.clone()));
        // }

        // if let Some(grid_draw_object) = &self.grid_draw_object {
        //     self.render_thread_mode
        //         .send_command(RenderCommand::DrawObject(grid_draw_object.clone()));
        // }

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
        // let mut draw_objects: Vec<_> = player_viewport.draw_objects.drain(..).collect();
        if let Some(grid_draw_object) = &player_viewport.grid_draw_object {
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
                window_id: player_viewport.window_id,
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

    pub fn present(&mut self, window_id: isize) {
        let mut draw_objects = self.draw_objects.entry(window_id).or_default().clone();
        if let Some(grid_draw_object) = &self.grid_draw_object {
            if window_id == self.main_window_id {
                draw_objects.push(grid_draw_object.clone());
            }
        }
        let virtual_texture_pass = if window_id == self.main_window_id {
            self.virtual_pass_handle.clone().map(|x| x.key())
        } else {
            None
        };
        if let Some(key) = virtual_texture_pass {
            self.render_thread_mode
                .send_command(RenderCommand::ClearVirtualTexturePass(key));
        }

        if let Some(shadow_depth_texture_handle) = if window_id == self.main_window_id {
            self.shadow_depth_texture_handle.clone()
        } else {
            None
        } {
            self.render_thread_mode
                .send_command(RenderCommand::ClearDepthTexture(ClearDepthTexture {
                    handle: *shadow_depth_texture_handle,
                }));
        }

        let player_viewport = self.player_viewports.get(0).unwrap();
        let player_viewport = player_viewport.borrow();

        self.render_thread_mode
            .send_command(RenderCommand::Present(PresentInfo {
                window_id,
                draw_objects,
                virtual_texture_pass,
                scene_viewport: player_viewport.scene_viewport.clone(),
            }));
        self.draw_objects.entry(window_id).or_default().clear();

        let pending_destroy_textures = ResourceManager::default().get_pending_destroy_textures();
        if !pending_destroy_textures.is_empty() {
            let pending_destroy_textures = pending_destroy_textures.iter().map(|x| **x).collect();
            self.render_thread_mode
                .send_command(RenderCommand::DestroyTextures(pending_destroy_textures));
        }
    }

    pub fn resize(&mut self, window_id: isize, surface_width: u32, surface_height: u32) {
        let virtual_texture_pass = if window_id == self.main_window_id {
            self.virtual_pass_handle.clone().map(|x| x.key())
        } else {
            None
        };
        if let Some(key) = virtual_texture_pass {
            self.render_thread_mode
                .send_command(RenderCommand::VirtualTexturePassResize(
                    VirtualTexturePassResize {
                        key,
                        surface_size: glam::uvec2(surface_width, surface_height),
                    },
                ));
        }
        self.render_thread_mode.send_command(RenderCommand::Resize(
            rs_render::command::ResizeInfo {
                width: surface_width,
                height: surface_height,
                window_id,
            },
        ));

        let player_viewport = self.player_viewports.get(0).unwrap().clone();
        let mut player_viewport = player_viewport.borrow_mut();
        player_viewport.size_changed(surface_width, surface_height, self);
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
            global_constants_resource: EBindingResource::Constants(*self.global_constants_handle),
            base_color_sampler_resource: EBindingResource::Sampler(*self.global_sampler_handle),
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
            global_constants_resource: EBindingResource::Constants(*self.global_constants_handle),
            base_color_sampler_resource: EBindingResource::Sampler(*self.global_sampler_handle),
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

        let object = MaterialDrawObject {
            id,
            vertex_buffers: vertex_buffer_handles,
            vertex_count: vertexes0.len() as u32,
            index_buffer: Some(index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            global_constants_resource: EBindingResource::Constants(*self.global_constants_handle),
            base_color_sampler_resource: EBindingResource::Sampler(*self.global_sampler_handle),
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
                *self
                    .shadow_depth_texture_handle
                    .clone()
                    .unwrap_or(self.default_textures.get_texture_handle()),
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

        let object = StaticMeshMaterialDrawObject {
            id,
            vertex_buffers: vertex_buffer_handles,
            vertex_count: vertexes0.len() as u32,
            index_buffer: Some(index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            global_constants_resource: EBindingResource::Constants(*self.global_constants_handle),
            base_color_sampler_resource: EBindingResource::Sampler(*self.global_sampler_handle),
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
                *self
                    .shadow_depth_texture_handle
                    .clone()
                    .unwrap_or(self.default_textures.get_texture_handle()),
            ),
        };
        EDrawObjectType::StaticMeshMaterial(object)
    }

    pub fn update_draw_object(&mut self, object: &mut EDrawObjectType) {
        match object {
            EDrawObjectType::Static(object) => {
                self.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
                if let Some(texture_url) = object.diffuse_texture_url.as_ref() {
                    if let Some(_) = self
                        .resource_manager
                        .get_virtual_texture_by_url(texture_url)
                    {
                        let virtual_texture_source_infos =
                            self.virtual_texture_source_infos.borrow();
                        let source = virtual_texture_source_infos.get(texture_url).unwrap();
                        {
                            let source = source.lock().unwrap();
                            let max_mips = rs_core_minimal::misc::calculate_max_mips(
                                source.get_size().min_element(),
                            );
                            let max_lod = max_mips
                                - self
                                    .settings
                                    .render_setting
                                    .virtual_texture_setting
                                    .tile_size
                                    .ilog2()
                                - 1;
                            object.constants.diffuse_texture_max_lod = max_lod;
                            object.constants.diffuse_texture_size = source.get_size().as_vec2();
                        }
                        object.constants.is_virtual_diffuse_texture = 1;
                        object.diffuse_texture_resource =
                            EBindingResource::Texture(*self.default_textures.get_texture_handle());
                    } else if let Some(base_texture_handle) =
                        self.resource_manager.get_texture_by_url(texture_url)
                    {
                        object.constants.is_virtual_diffuse_texture = 0;
                        object.diffuse_texture_resource =
                            EBindingResource::Texture(*base_texture_handle);
                    }
                }
            }
            EDrawObjectType::Skin(object) => {
                self.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
                if let Some(texture_url) = object.diffuse_texture_url.as_ref() {
                    if let Some(_) = self
                        .resource_manager
                        .get_virtual_texture_by_url(texture_url)
                    {
                        let virtual_texture_source_infos =
                            self.virtual_texture_source_infos.borrow();
                        let source = virtual_texture_source_infos.get(texture_url).unwrap();
                        {
                            let source = source.lock().unwrap();
                            let max_mips = rs_core_minimal::misc::calculate_max_mips(
                                source.get_size().min_element(),
                            );
                            let max_lod = max_mips
                                - self
                                    .settings
                                    .render_setting
                                    .virtual_texture_setting
                                    .tile_size
                                    .ilog2()
                                - 1;
                            object.constants.diffuse_texture_max_lod = max_lod;
                            object.constants.diffuse_texture_size = source.get_size().as_vec2();
                        }
                        object.constants.is_virtual_diffuse_texture = 1;
                        object.diffuse_texture_resource =
                            EBindingResource::Texture(*self.default_textures.get_texture_handle());
                    } else if let Some(base_texture_handle) =
                        self.resource_manager.get_texture_by_url(texture_url)
                    {
                        object.constants.is_virtual_diffuse_texture = 0;
                        object.diffuse_texture_resource =
                            EBindingResource::Texture(*base_texture_handle);
                    }
                }
            }
            EDrawObjectType::SkinMaterial(object) => {
                self.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
                let material_info = object.material.borrow().get_material_info().clone();
                let map_textures = &material_info
                    .get(&MaterialOptions { is_skin: true })
                    .unwrap()
                    .map_textures;
                for virtual_texture_url in &material_info
                    .get(&MaterialOptions { is_skin: true })
                    .unwrap()
                    .virtual_textures
                {
                    let virtual_texture_source_infos = self.virtual_texture_source_infos.borrow();
                    let source = virtual_texture_source_infos
                        .get(virtual_texture_url)
                        .unwrap();
                    {
                        let source = source.lock().unwrap();
                        let max_mips = rs_core_minimal::misc::calculate_max_mips(
                            source.get_size().min_element(),
                        );
                        let max_lod = max_mips
                            - self
                                .settings
                                .render_setting
                                .virtual_texture_setting
                                .tile_size
                                .ilog2()
                            - 1;
                        object.virtual_texture_constants.virtual_texture_max_lod = max_lod;
                        object.virtual_texture_constants.virtual_texture_size =
                            source.get_size().as_vec2();
                    }
                }
                self.update_buffer(
                    object.skin_constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.skin_constants),
                );
                self.update_buffer(
                    object.virtual_texture_constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.virtual_texture_constants),
                );

                let mut binding_resources: Vec<EBindingResource> =
                    Vec::with_capacity(map_textures.len());
                for map_texture in map_textures {
                    if let Some(handle) = self
                        .resource_manager
                        .get_texture_by_url(&map_texture.texture_url)
                    {
                        binding_resources.push(EBindingResource::Texture(*handle));
                    } else {
                        log::trace!("Can not find {}", map_texture.texture_url.to_string());
                    }
                }
                assert_eq!(binding_resources.len(), map_textures.len());
                object.user_textures_resources = binding_resources;
                let ibl_textures = ResourceManager::default().get_ibl_textures();
                let Some((_, ibl_textures)) = ibl_textures.iter().find(|x| {
                    let url = x.0;
                    url.scheme() != BUILT_IN_RESOURCE
                }) else {
                    return;
                };
                object.brdflut_texture_resource = EBindingResource::Texture(*ibl_textures.brdflut);
                object.pre_filter_cube_map_texture_resource =
                    EBindingResource::Texture(*ibl_textures.pre_filter_cube_map);
                object.irradiance_texture_resource =
                    EBindingResource::Texture(*ibl_textures.irradiance);
            }
            EDrawObjectType::StaticMeshMaterial(object) => {
                self.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
                let material_info = object.material.borrow().get_material_info().clone();
                let map_textures = &material_info
                    .get(&MaterialOptions { is_skin: true })
                    .unwrap()
                    .map_textures;
                for virtual_texture_url in &material_info
                    .get(&MaterialOptions { is_skin: true })
                    .unwrap()
                    .virtual_textures
                {
                    let virtual_texture_source_infos = self.virtual_texture_source_infos.borrow();
                    let source = virtual_texture_source_infos
                        .get(virtual_texture_url)
                        .unwrap();
                    {
                        let source = source.lock().unwrap();
                        let max_mips = rs_core_minimal::misc::calculate_max_mips(
                            source.get_size().min_element(),
                        );
                        let max_lod = max_mips
                            - self
                                .settings
                                .render_setting
                                .virtual_texture_setting
                                .tile_size
                                .ilog2()
                            - 1;
                        object.virtual_texture_constants.virtual_texture_max_lod = max_lod;
                        object.virtual_texture_constants.virtual_texture_size =
                            source.get_size().as_vec2();
                    }
                }

                self.update_buffer(
                    object.virtual_texture_constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.virtual_texture_constants),
                );

                let mut binding_resources: Vec<EBindingResource> =
                    Vec::with_capacity(map_textures.len());
                for map_texture in map_textures {
                    if let Some(handle) = self
                        .resource_manager
                        .get_texture_by_url(&map_texture.texture_url)
                    {
                        binding_resources.push(EBindingResource::Texture(*handle));
                    } else {
                        log::trace!("Can not find {}", map_texture.texture_url.to_string());
                    }
                }
                assert_eq!(binding_resources.len(), map_textures.len());
                object.user_textures_resources = binding_resources;
                let ibl_textures = ResourceManager::default().get_ibl_textures();
                let Some((_, ibl_textures)) = ibl_textures.iter().find(|x| {
                    let url = x.0;
                    url.scheme() != BUILT_IN_RESOURCE
                }) else {
                    return;
                };
                object.brdflut_texture_resource = EBindingResource::Texture(*ibl_textures.brdflut);
                object.pre_filter_cube_map_texture_resource =
                    EBindingResource::Texture(*ibl_textures.pre_filter_cube_map);
                object.irradiance_texture_resource =
                    EBindingResource::Texture(*ibl_textures.irradiance);
            }
            EDrawObjectType::Custom(_) => {}
        }
    }

    pub fn draw2(&mut self, draw_object: &EDrawObjectType) {
        match draw_object {
            EDrawObjectType::Static(static_objcet) => {
                let static_objcet = static_objcet.clone();
                let draw_object = DrawObject::new(
                    static_objcet.id,
                    static_objcet.vertex_buffers.iter().map(|x| **x).collect(),
                    static_objcet.vertex_count,
                    EPipelineType::Builtin(EBuiltinPipelineType::StaticMeshPhong),
                    static_objcet.index_buffer.clone().map(|x| *x),
                    static_objcet.index_count,
                    vec![
                        vec![
                            static_objcet.global_constants_resource,
                            static_objcet.base_color_sampler_resource,
                            static_objcet.physical_texture_resource,
                            static_objcet.page_table_texture_resource,
                        ],
                        vec![
                            static_objcet.diffuse_texture_resource,
                            static_objcet.specular_texture_resource,
                        ],
                        vec![static_objcet.constants_resource],
                    ],
                );

                self.draw_objects
                    .entry(static_objcet.window_id)
                    .or_default()
                    .push(draw_object);
            }
            EDrawObjectType::Skin(skin_objcet) => {
                let skin_objcet = skin_objcet.clone();

                let draw_object = DrawObject::new(
                    skin_objcet.id,
                    skin_objcet.vertex_buffers.iter().map(|x| **x).collect(),
                    skin_objcet.vertex_count,
                    EPipelineType::Builtin(EBuiltinPipelineType::SkinMeshPhong),
                    skin_objcet.index_buffer.clone().map(|x| *x),
                    skin_objcet.index_count,
                    vec![
                        vec![
                            skin_objcet.global_constants_resource,
                            skin_objcet.base_color_sampler_resource,
                            skin_objcet.physical_texture_resource,
                            skin_objcet.page_table_texture_resource,
                        ],
                        vec![
                            skin_objcet.diffuse_texture_resource,
                            skin_objcet.specular_texture_resource,
                        ],
                        vec![skin_objcet.constants_resource],
                    ],
                );
                self.draw_objects
                    .entry(skin_objcet.window_id)
                    .or_default()
                    .push(draw_object);
            }
            EDrawObjectType::SkinMaterial(skin_objcet) => {
                let skin_objcet = skin_objcet.clone();
                let material = skin_objcet.material.borrow();
                if let Some(pipeline_handle) = material.get_pipeline_handle() {
                    let mut draw_object = DrawObject::new(
                        skin_objcet.id,
                        skin_objcet.vertex_buffers.iter().map(|x| **x).collect(),
                        skin_objcet.vertex_count,
                        EPipelineType::Material(MaterialPipelineType {
                            handle: *pipeline_handle,
                            options: MaterialOptions { is_skin: true },
                        }),
                        skin_objcet.index_buffer.clone().map(|x| *x),
                        skin_objcet.index_count,
                        vec![
                            vec![
                                skin_objcet.global_constants_resource.clone(),
                                skin_objcet.base_color_sampler_resource,
                                skin_objcet.physical_texture_resource,
                                skin_objcet.page_table_texture_resource,
                                skin_objcet.brdflut_texture_resource,
                                skin_objcet.pre_filter_cube_map_texture_resource,
                                skin_objcet.irradiance_texture_resource,
                                skin_objcet.shadow_map_texture_resource,
                            ],
                            vec![
                                skin_objcet.constants_resource.clone(),
                                skin_objcet.skin_constants_resource.clone(),
                                skin_objcet.virtual_texture_constants_resource,
                            ],
                            skin_objcet.user_textures_resources,
                        ],
                    );
                    draw_object.virtual_pass_set = Some(VirtualPassSet {
                        vertex_buffers: vec![
                            *skin_objcet.vertex_buffers[0],
                            *skin_objcet.vertex_buffers[2],
                        ],
                        binding_resources: vec![
                            vec![skin_objcet.global_constants_resource.clone()],
                            vec![
                                skin_objcet.constants_resource.clone(),
                                skin_objcet.skin_constants_resource.clone(),
                            ],
                        ],
                    });
                    if let Some(handle) = self.shadow_depth_texture_handle.clone() {
                        draw_object.shadow_mapping = Some(ShadowMapping {
                            vertex_buffers: vec![
                                *skin_objcet.vertex_buffers[0],
                                *skin_objcet.vertex_buffers[2],
                            ],
                            depth_texture_handle: *handle,
                            binding_resources: vec![vec![
                                skin_objcet.global_constants_resource.clone(),
                                skin_objcet.constants_resource.clone(),
                                skin_objcet.skin_constants_resource.clone(),
                            ]],
                            is_skin: true,
                        });
                    }
                    self.draw_objects
                        .entry(skin_objcet.window_id)
                        .or_default()
                        .push(draw_object);
                }
            }
            EDrawObjectType::StaticMeshMaterial(static_mesh_draw_objcet) => {
                let static_mesh_draw_objcet = static_mesh_draw_objcet.clone();
                let material = static_mesh_draw_objcet.material.borrow();
                if let Some(pipeline_handle) = material.get_pipeline_handle() {
                    let mut draw_object = DrawObject::new(
                        static_mesh_draw_objcet.id,
                        static_mesh_draw_objcet
                            .vertex_buffers
                            .iter()
                            .map(|x| **x)
                            .collect(),
                        static_mesh_draw_objcet.vertex_count,
                        EPipelineType::Material(MaterialPipelineType {
                            handle: *pipeline_handle,
                            options: MaterialOptions { is_skin: false },
                        }),
                        static_mesh_draw_objcet.index_buffer.clone().map(|x| *x),
                        static_mesh_draw_objcet.index_count,
                        vec![
                            vec![
                                static_mesh_draw_objcet.global_constants_resource.clone(),
                                static_mesh_draw_objcet.base_color_sampler_resource,
                                static_mesh_draw_objcet.physical_texture_resource,
                                static_mesh_draw_objcet.page_table_texture_resource,
                                static_mesh_draw_objcet.brdflut_texture_resource,
                                static_mesh_draw_objcet.pre_filter_cube_map_texture_resource,
                                static_mesh_draw_objcet.irradiance_texture_resource,
                                static_mesh_draw_objcet.shadow_map_texture_resource,
                            ],
                            vec![
                                static_mesh_draw_objcet.constants_resource.clone(),
                                static_mesh_draw_objcet.virtual_texture_constants_resource,
                            ],
                            static_mesh_draw_objcet.user_textures_resources,
                        ],
                    );
                    draw_object.virtual_pass_set = Some(VirtualPassSet {
                        vertex_buffers: vec![*static_mesh_draw_objcet.vertex_buffers[0]],
                        binding_resources: vec![
                            vec![static_mesh_draw_objcet.global_constants_resource.clone()],
                            vec![static_mesh_draw_objcet.constants_resource.clone()],
                        ],
                    });
                    if let Some(handle) = self.shadow_depth_texture_handle.clone() {
                        draw_object.shadow_mapping = Some(ShadowMapping {
                            vertex_buffers: vec![*static_mesh_draw_objcet.vertex_buffers[0]],
                            depth_texture_handle: *handle,
                            binding_resources: vec![vec![
                                static_mesh_draw_objcet.global_constants_resource.clone(),
                                static_mesh_draw_objcet.constants_resource.clone(),
                            ]],
                            is_skin: false,
                        });
                    }
                    self.draw_objects
                        .entry(static_mesh_draw_objcet.window_id)
                        .or_default()
                        .push(draw_object);
                }
            }
            EDrawObjectType::Custom(custom_objcet) => {
                self.draw_objects
                    .entry(custom_objcet.window_id)
                    .or_default()
                    .push(custom_objcet.draw_object.clone());
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn process_device_event(&mut self, device_event: winit::event::DeviceEvent) {
        match device_event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                DefaultCameraInputEventHandle::mouse_motion_handle(
                    &mut self.camera,
                    delta,
                    self.input_mode,
                    self.state.camera_motion_speed,
                );
            }
            _ => {}
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn process_keyboard_input(
        &mut self,
        _device_id: winit::event::DeviceId,
        event: winit::event::KeyEvent,
        _is_synthetic: bool,
    ) {
        let winit::keyboard::PhysicalKey::Code(virtual_keycode) = event.physical_key else {
            return;
        };
        self.state
            .virtual_key_code_states
            .insert(virtual_keycode, event.state);
    }

    pub fn camera_did_update(
        &mut self,
        view: glam::Mat4,
        projection: glam::Mat4,
        world_location: glam::Vec3,
    ) {
        self.global_constants.view = view;
        self.global_constants.projection = projection;
        self.global_constants.view_projection =
            self.global_constants.projection * self.global_constants.view;
        self.global_constants.view_position = world_location;
    }

    fn update_global_constants(&mut self) {
        let command = RenderCommand::UpdateBuffer(UpdateBuffer {
            handle: *self.global_constants_handle,
            data: rs_foundation::cast_to_raw_buffer(&vec![self.global_constants]).to_vec(),
        });
        self.render_thread_mode.send_command(command);
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

    pub fn set_input_mode(&mut self, input_mode: EInputMode) {
        self.input_mode = input_mode;
    }

    pub fn get_input_mode(&self) -> EInputMode {
        self.input_mode
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

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
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
        id: u32,
        global_constants_handle: crate::handle::BufferHandle,
    ) -> DrawObject {
        let resource_manager = ResourceManager::default();
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
    ) -> DrawObject {
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
        let draw_object = DrawObject::new(
            id,
            vertex_buffer_handles.iter().map(|x| **x).collect(),
            vertexes0.len() as u32,
            EPipelineType::Builtin(EBuiltinPipelineType::Grid),
            Some(*index_buffer_handle),
            Some(grid_data.indices.len() as u32),
            vec![vec![EBindingResource::Constants(*global_constants_handle)]],
        );
        draw_object
    }

    pub fn set_camera_movement_speed(&mut self, new_speed: f32) {
        self.state.camera_movement_speed = new_speed;
    }

    pub fn get_camera_movement_speed(&mut self) -> f32 {
        self.state.camera_movement_speed
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

    pub fn set_debug_shading(&mut self, ty: EDebugShadingType) {
        self.global_constants.set_shading_type(ty);
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

    pub fn update_light(&mut self, light: &mut DirectionalLight) {
        self.global_constants.light_space_matrix = light.get_light_space_matrix();
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

    pub fn on_antialias_type_changed(
        &mut self,
        antialias_type: rs_core_minimal::settings::EAntialiasType,
    ) {
        for player_viewport in self.player_viewports.clone() {
            let mut player_viewport = player_viewport.borrow_mut();
            match antialias_type {
                rs_core_minimal::settings::EAntialiasType::None => {
                    player_viewport.disable_antialias();
                }
                rs_core_minimal::settings::EAntialiasType::FXAA => {
                    player_viewport.enable_fxaa(self);
                }
                rs_core_minimal::settings::EAntialiasType::MSAA => {
                    player_viewport.enable_msaa(self);
                }
            }
        }
    }

    pub fn get_global_constants_handle(&self) -> crate::handle::BufferHandle {
        self.global_constants_handle.clone()
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
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.logger.flush();
    }
}
