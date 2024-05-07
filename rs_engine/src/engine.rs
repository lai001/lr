use crate::camera::Camera;
#[cfg(not(target_os = "android"))]
use crate::camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle};
use crate::console_cmd::ConsoleCmd;
use crate::content::content_file_type::EContentFileType;
use crate::drawable::{
    EDrawObjectType, MaterialDrawObject, SkinMeshDrawObject, StaticMeshDrawObject,
};
use crate::error::Result;
use crate::input_mode::EInputMode;
use crate::render_thread_mode::ERenderThreadMode;
use crate::scene_node::EComponentType;
use crate::{logger::Logger, resource_manager::ResourceManager};
use rs_artifact::artifact::ArtifactReader;
use rs_artifact::content_type::EContentType;
use rs_artifact::resource_info::ResourceInfo;
use rs_artifact::resource_type::EResourceType;
use rs_core_minimal::primitive_data::PrimitiveData;
use rs_core_minimal::settings::Settings;
use rs_foundation::new::{
    MultipleThreadMut, MultipleThreadMutType, SingleThreadMut, SingleThreadMutType,
};
use rs_render::bake_info::BakeInfo;
use rs_render::command::{
    BufferCreateInfo, CreateBuffer, CreateIBLBake, CreateMaterialRenderPipeline, CreateSampler,
    CreateTexture, CreateVirtualTexture, DrawObject, EBindingResource, ETextureType,
    InitTextureData, RenderCommand, TextureDescriptorCreateInfo, UpdateBuffer, UploadPrebakeIBL,
};
use rs_render::egui_render::EGUIRenderOutput;
use rs_render::global_uniform;
use rs_render::renderer::Renderer;
use rs_render::vertex_data_type::mesh_vertex::MeshVertex0;
use rs_render::view_mode::EViewModeType;
use rs_render::virtual_texture_source::TVirtualTextureSource;
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::zip;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

struct State {
    camera_movement_speed: f32,
    camera_motion_speed: f32,
    #[cfg(not(target_os = "android"))]
    virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            camera_movement_speed: 0.01,
            camera_motion_speed: 0.1,
            #[cfg(not(target_os = "android"))]
            virtual_key_code_states: Default::default(),
        }
    }
}

pub struct Engine {
    render_thread_mode: ERenderThreadMode,
    resource_manager: ResourceManager,
    logger: Logger,
    level: Option<crate::content::level::Level>,
    draw_objects: Vec<DrawObject>,
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
    virtual_texture_source_infos:
        HashMap<url::Url, MultipleThreadMutType<Box<dyn TVirtualTextureSource>>>,
    console_cmds: SingleThreadMutType<HashMap<String, SingleThreadMutType<ConsoleCmd>>>,
    grid_draw_object: Option<DrawObject>,
    content_files: HashMap<url::Url, EContentFileType>,
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
        let settings: Settings;
        if let Some(artifact_reader) = &mut artifact_reader {
            settings = artifact_reader.get_artifact_file_header().settings.clone();
            artifact_reader.check_assets().expect("Valid");
        } else {
            settings = Settings::default();
        }

        #[cfg(target_os = "android")]
        (|| {
            if settings.render_setting.backends == rs_core_minimal::settings::Backends::DX12 {
                panic!("Not supported render backend on android platform.");
            }
        })();

        log::trace!("{:?}", settings);
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

        let mut draw_objects: Vec<DrawObject> = Vec::new();

        let mut render_thread_mode = ERenderThreadMode::from(renderer, true);

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
        let mut level: Option<crate::content::level::Level> = None;
        let mut draw_object_id: u32 = 0;
        (|| {
            let Some(url) = Self::find_first_level(&mut resource_manager) else {
                return;
            };
            let Ok(_level) = resource_manager.get_level(&url) else {
                return;
            };
            level = Some(_level);
        })();

        #[cfg(feature = "editor")]
        let grid_draw_object = (|| {
            draw_object_id += 1;
            Some(Self::create_grid_draw_object(
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

        let engine = Engine {
            render_thread_mode,
            resource_manager,
            logger,
            level,
            draw_objects,
            camera,
            state: State::default(),
            settings,
            draw_object_id,
            game_time: std::time::Instant::now(),
            game_time_sec: 0.0,
            input_mode,
            global_constants,
            global_constants_handle: global_constants_handle.clone(),
            global_sampler_handle: global_sampler_handle.clone(),
            virtual_texture_source_infos: HashMap::new(),
            console_cmds: SingleThreadMut::new(HashMap::new()),
            grid_draw_object,
            content_files: Self::collect_content_files(),
        };

        Ok(engine)
    }

    fn collect_content_files() -> HashMap<url::Url, EContentFileType> {
        let resource_manager = ResourceManager::default();
        let mut files: HashMap<url::Url, EContentFileType> = HashMap::new();
        if let Ok(resource_map) = resource_manager.get_resource_map() {
            for (url, v) in resource_map.iter() {
                match v.resource_type {
                    EResourceType::Content(content_ty) => match content_ty {
                        EContentType::StaticMesh => {}
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
                        EContentType::IBL => {}
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
                                material_content
                                    .borrow_mut()
                                    .set_pipeline_handle(pipeline_handle);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let Some(level) = self.level.as_mut() else {
            return;
        };
        for actor in level.actors.clone() {
            let mut actor = actor.borrow_mut();
            let root_scene_node = &mut actor.scene_node;
            let mut root_scene_node = root_scene_node.borrow_mut();
            match &mut root_scene_node.component {
                crate::scene_node::EComponentType::SceneComponent(_) => todo!(),
                crate::scene_node::EComponentType::StaticMeshComponent(_) => todo!(),
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
    }

    fn find_first_level(resource_manager: &mut ResourceManager) -> Option<url::Url> {
        let Ok(resource_map) = resource_manager.get_resource_map() else {
            return None;
        };
        for (k, v) in resource_map {
            if k.scheme() != "content" {
                continue;
            }
            let Some(host) = k.host() else {
                continue;
            };
            match host {
                url::Host::Domain(host) => {
                    if host == "level" {
                        return Some(v.url);
                    }
                }
                _ => {}
            }
        }
        return None;
    }

    pub fn get_resource_map(&self) -> Result<HashMap<url::Url, ResourceInfo>> {
        self.resource_manager.get_resource_map()
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

        if let Some(level) = self.level.as_ref() {
            for actor in level.actors.clone() {
                match &mut actor.borrow_mut().scene_node.borrow_mut().component {
                    EComponentType::SceneComponent(_) => todo!(),
                    EComponentType::StaticMeshComponent(_) => todo!(),
                    EComponentType::SkeletonMeshComponent(skeleton_mesh_component) => {
                        let mut skeleton_mesh_component = skeleton_mesh_component.borrow_mut();
                        skeleton_mesh_component.update(self.get_game_time(), self);
                        for draw_object in skeleton_mesh_component.get_draw_objects() {
                            self.draw2(draw_object);
                        }
                    }
                }
            }
        }

        for draw_object in &self.draw_objects {
            self.render_thread_mode
                .send_command(RenderCommand::DrawObject(draw_object.clone()));
        }

        if let Some(grid_draw_object) = &self.grid_draw_object {
            self.render_thread_mode
                .send_command(RenderCommand::DrawObject(grid_draw_object.clone()));
        }

        self.render_thread_mode
            .send_command(RenderCommand::UiOutput(gui_render_output));
    }

    pub fn present(&mut self, window_id: isize) {
        self.render_thread_mode
            .send_command(RenderCommand::Present(window_id));
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
    ) -> Result<()>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        self.render_thread_mode
            .set_new_window(window_id, window, surface_width, surface_height)
    }

    fn create_draw_object_from_mesh_internal(
        render_thread_mode: &mut ERenderThreadMode,
        resource_manager: &mut ResourceManager,
        vertex_buffers: Vec<(Option<&str>, &[u8])>,
        vertex_count: u32,
        indexes: &[u32],
        id: u32,
        name: Option<String>,
        render_pipeline: String,
        global_binding_resources: Vec<EBindingResource>,
    ) -> DrawObject {
        let index_buffer_handle = resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!(
                "IndexBuffer.{}",
                name.clone().unwrap_or("".to_string())
            )),
            contents: rs_foundation::cast_to_raw_buffer(&indexes).to_vec(),
            usage: wgpu::BufferUsages::INDEX,
        };
        let create_buffer = CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        render_thread_mode.send_command(message);
        let mut vertex_buffer_handles: Vec<u64> = Vec::with_capacity(vertex_buffers.len());
        for (name, vertex_buffer) in vertex_buffers {
            let vertex_buffer_handle = resource_manager.next_buffer();
            let buffer_create_info = BufferCreateInfo {
                label: Some(format!("VertexBuffer.{}", name.unwrap_or(""))),
                contents: vertex_buffer.to_vec(),
                usage: wgpu::BufferUsages::VERTEX,
            };
            let create_buffer = CreateBuffer {
                handle: *vertex_buffer_handle,
                buffer_create_info,
            };
            let message = RenderCommand::CreateBuffer(create_buffer);
            render_thread_mode.send_command(message);
            vertex_buffer_handles.push(*vertex_buffer_handle);
        }

        let draw_object = DrawObject {
            id,
            vertex_buffers: vertex_buffer_handles,
            vertex_count,
            index_buffer: Some(*index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            render_pipeline,
            binding_resources: vec![],
            global_binding_resources,
            vt_binding_resources: vec![],
        };
        draw_object
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
            global_binding_resources: vec![
                EBindingResource::Constants(*self.global_constants_handle),
                EBindingResource::Sampler(*self.global_sampler_handle),
            ],
            vt_binding_resources: vec![EBindingResource::Constants(*constants_buffer_handle)],
            binding_resources: vec![
                vec![
                    EBindingResource::Texture(ETextureType::None),
                    EBindingResource::Texture(ETextureType::None),
                ],
                vec![EBindingResource::Constants(*constants_buffer_handle)],
            ],
            render_pipeline: rs_render::renderer::STATIC_MESH_RENDER_PIPELINE.to_string(),
            constants: Default::default(),
            diffuse_texture_url: Default::default(),
            specular_texture_url: Default::default(),
            constants_buffer_handle,
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
            global_binding_resources: vec![
                EBindingResource::Constants(*self.global_constants_handle),
                EBindingResource::Sampler(*self.global_sampler_handle),
            ],
            vt_binding_resources: vec![EBindingResource::Constants(*constants_buffer_handle)],
            binding_resources: vec![
                vec![
                    EBindingResource::Texture(ETextureType::None),
                    EBindingResource::Texture(ETextureType::None),
                ],
                vec![EBindingResource::Constants(*constants_buffer_handle)],
            ],
            render_pipeline: rs_render::renderer::SKIN_MESH_RENDER_PIPELINE.to_string(),
            constants: Default::default(),
            diffuse_texture_url: Default::default(),
            specular_texture_url: Default::default(),
            constants_buffer_handle,
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

        let constants_buffer_handle = self.resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("rs.{}.Constants", name.clone())),
            contents: rs_foundation::cast_any_as_u8_slice(
                &rs_render::render_pipeline::material_pipeline::Constants::default(),
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

        let object = MaterialDrawObject {
            id,
            vertex_buffers: vertex_buffer_handles,
            vertex_count: vertexes0.len() as u32,
            index_buffer: Some(index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            global_binding_resources: vec![
                EBindingResource::Constants(*self.global_constants_handle),
                EBindingResource::Sampler(*self.global_sampler_handle),
            ],
            vt_binding_resources: vec![EBindingResource::Constants(*constants_buffer_handle)],
            binding_resources: vec![vec![EBindingResource::Constants(*constants_buffer_handle)]],
            material,
            constants: Default::default(),
            constants_buffer_handle,
        };
        EDrawObjectType::SkinMaterial(object)
    }

    pub fn update_draw_object(&mut self, object: &mut EDrawObjectType) {
        match object {
            EDrawObjectType::Static(object) => {
                self.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
                if let Some(texture_url) = object.diffuse_texture_url.as_ref() {
                    if let Some(virtual_texture_handle) = self
                        .resource_manager
                        .get_virtual_texture_by_url(texture_url)
                    {
                        let source = self.virtual_texture_source_infos.get(texture_url).unwrap();
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
                        object.binding_resources[0][0] = EBindingResource::Texture(
                            ETextureType::Virtual(*virtual_texture_handle),
                        );
                    } else if let Some(base_texture_handle) =
                        self.resource_manager.get_texture_by_url(texture_url)
                    {
                        object.constants.is_virtual_diffuse_texture = 0;
                        object.binding_resources[0][0] =
                            EBindingResource::Texture(ETextureType::Base(*base_texture_handle));
                    }
                }
            }
            EDrawObjectType::Skin(object) => {
                self.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
                if let Some(texture_url) = object.diffuse_texture_url.as_ref() {
                    if let Some(virtual_texture_handle) = self
                        .resource_manager
                        .get_virtual_texture_by_url(texture_url)
                    {
                        let source = self.virtual_texture_source_infos.get(texture_url).unwrap();
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
                        object.binding_resources[0][0] = EBindingResource::Texture(
                            ETextureType::Virtual(*virtual_texture_handle),
                        );
                    } else if let Some(base_texture_handle) =
                        self.resource_manager.get_texture_by_url(texture_url)
                    {
                        object.constants.is_virtual_diffuse_texture = 0;
                        object.binding_resources[0][0] =
                            EBindingResource::Texture(ETextureType::Base(*base_texture_handle));
                    }
                }
            }
            EDrawObjectType::SkinMaterial(object) => {
                self.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
                let map_textures = object.material.borrow().get_map_textures().clone();
                let mut binding_resources: Vec<EBindingResource> =
                    Vec::with_capacity(map_textures.len());
                for map_texture in map_textures {
                    if let Some(handle) = self
                        .resource_manager
                        .get_texture_by_url(&map_texture.texture_url)
                    {
                        binding_resources
                            .push(EBindingResource::Texture(ETextureType::Base(*handle)));
                    }
                }
                if let Some(textures) = object.binding_resources.get_mut(1) {
                    *textures = binding_resources;
                } else {
                    object.binding_resources.push(binding_resources);
                }
            }
        }
    }

    pub fn draw(&mut self, draw_object: DrawObject) {
        self.render_thread_mode
            .send_command(RenderCommand::DrawObject(draw_object));
    }

    pub fn draw2(&mut self, draw_object: &EDrawObjectType) {
        match draw_object {
            EDrawObjectType::Static(static_objcet) => {
                let draw_object = DrawObject {
                    id: static_objcet.id,
                    vertex_buffers: static_objcet.vertex_buffers.iter().map(|x| **x).collect(),
                    vertex_count: static_objcet.vertex_count,
                    index_buffer: static_objcet.index_buffer.clone().map(|x| *x),
                    index_count: static_objcet.index_count,
                    global_binding_resources: static_objcet.global_binding_resources.clone(),
                    vt_binding_resources: static_objcet.vt_binding_resources.clone(),
                    binding_resources: static_objcet.binding_resources.clone(),
                    render_pipeline: static_objcet.render_pipeline.clone(),
                };
                self.render_thread_mode
                    .send_command(RenderCommand::DrawObject(draw_object));
            }
            EDrawObjectType::Skin(skin_objcet) => {
                let draw_object = DrawObject {
                    id: skin_objcet.id,
                    vertex_buffers: skin_objcet.vertex_buffers.iter().map(|x| **x).collect(),
                    vertex_count: skin_objcet.vertex_count,
                    index_buffer: skin_objcet.index_buffer.clone().map(|x| *x),
                    index_count: skin_objcet.index_count,
                    global_binding_resources: skin_objcet.global_binding_resources.clone(),
                    vt_binding_resources: skin_objcet.vt_binding_resources.clone(),
                    binding_resources: skin_objcet.binding_resources.clone(),
                    render_pipeline: skin_objcet.render_pipeline.clone(),
                };
                self.render_thread_mode
                    .send_command(RenderCommand::DrawObject(draw_object));
            }
            EDrawObjectType::SkinMaterial(skin_objcet) => {
                if let Some(pipeline_handle) = skin_objcet.material.borrow().get_pipeline_handle() {
                    let render_pipeline =
                        rs_render::shader_library::ShaderLibrary::get_material_shader_name(
                            *pipeline_handle,
                        );
                    let draw_object = DrawObject {
                        id: skin_objcet.id,
                        vertex_buffers: skin_objcet.vertex_buffers.iter().map(|x| **x).collect(),
                        vertex_count: skin_objcet.vertex_count,
                        index_buffer: skin_objcet.index_buffer.clone().map(|x| *x),
                        index_count: skin_objcet.index_count,
                        global_binding_resources: skin_objcet.global_binding_resources.clone(),
                        vt_binding_resources: skin_objcet.vt_binding_resources.clone(),
                        binding_resources: skin_objcet.binding_resources.clone(),
                        render_pipeline,
                    };
                    self.render_thread_mode
                        .send_command(RenderCommand::DrawObject(draw_object));
                }
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
        device_id: winit::event::DeviceId,
        event: winit::event::KeyEvent,
        is_synthetic: bool,
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

    pub fn create_texture_from_path(
        &mut self,
        path: &Path,
        url: url::Url,
    ) -> Result<crate::handle::TextureHandle> {
        let dynamic_image =
            image::open(path).map_err(|err| crate::error::Error::ImageError(err, None))?;
        let image = match dynamic_image {
            image::DynamicImage::ImageRgba8(image) => image,
            x => x.to_rgba8(),
        };
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

    pub fn create_virtual_texture_source(
        &mut self,
        url: url::Url,
        source: Box<dyn TVirtualTextureSource>,
    ) {
        let ref_source = MultipleThreadMut::new(source);
        self.virtual_texture_source_infos
            .insert(url.clone(), ref_source.clone());
        let handle = self.resource_manager.next_virtual_texture(url.clone());
        let command = CreateVirtualTexture {
            handle: *handle,
            source: ref_source.clone(),
        };
        let render_command = RenderCommand::CreateVirtualTextureSource(command);
        self.render_thread_mode.send_command(render_command);
    }

    pub fn send_render_task(&mut self, task: rs_render::command::TaskType) {
        let render_command = RenderCommand::Task(task);
        self.render_thread_mode.send_command(render_command);
    }

    pub fn ibl_bake<P: AsRef<Path>>(
        &mut self,
        path: P,
        url: url::Url,
        bake_info: BakeInfo,
        save_dir: Option<P>,
    ) {
        let handle = self.resource_manager.next_texture(url.clone());
        let render_command = RenderCommand::CreateIBLBake(CreateIBLBake {
            handle: *handle,
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
        let handle = self.resource_manager.next_texture(url.clone());
        let render_command = RenderCommand::UploadPrebakeIBL(UploadPrebakeIBL {
            handle: *handle,
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
    fn create_grid_draw_object(
        id: u32,
        resource_manager: ResourceManager,
        render_thread_mode: &mut ERenderThreadMode,
        global_constants_handle: crate::handle::BufferHandle,
    ) -> DrawObject {
        let grid_data = PrimitiveData::quad();
        let name = "Grid";
        let index_buffer_handle = resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some(format!("rs.IndexBuffer.{}", name.clone())),
            contents: rs_foundation::cast_to_raw_buffer(&grid_data.indices).to_vec(),
            usage: wgpu::BufferUsages::INDEX,
        };
        let create_buffer = CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        render_thread_mode.send_command(message);

        let mut vertexes0: Vec<MeshVertex0> = vec![];

        for (position, tex_coord) in zip(&grid_data.vertex_positions, &grid_data.vertex_tex_coords)
        {
            vertexes0.push(MeshVertex0 {
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

        DrawObject {
            id,
            vertex_buffers: vertex_buffer_handles.iter().map(|x| **x).collect(),
            vertex_count: vertexes0.len() as u32,
            index_buffer: Some(*index_buffer_handle),
            index_count: Some(grid_data.indices.len() as u32),
            global_binding_resources: vec![EBindingResource::Constants(*global_constants_handle)],
            vt_binding_resources: vec![],
            binding_resources: vec![],
            render_pipeline: rs_render::renderer::GRID_RENDER_PIPELINE.to_string(),
        }
    }

    pub fn set_camera_movement_speed(&mut self, new_speed: f32) {
        self.state.camera_movement_speed = new_speed;
    }

    pub fn get_camera_movement_speed(&mut self) -> f32 {
        self.state.camera_movement_speed
    }

    pub fn create_material(
        &mut self,
        shader_code: String,
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
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.logger.flush();
    }
}
