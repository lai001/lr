use crate::camera::Camera;
#[cfg(not(target_os = "android"))]
use crate::camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle};
use crate::error::Result;
use crate::render_thread_mode::{ERenderThreadMode, MultipleThreadRenderer};
use crate::{
    logger::{Logger, LoggerConfiguration},
    resource_manager::ResourceManager,
};
use rs_artifact::artifact::ArtifactReader;
use rs_artifact::level::ENodeType;
use rs_artifact::resource_info::ResourceInfo;
use rs_render::command::{
    BufferCreateInfo, CreateBuffer, CreateTexture, DrawObject, EMaterialType, InitTextureData,
    PhongMaterial, RenderCommand, TextureDescriptorCreateInfo,
};
use rs_render::egui_render::EGUIRenderOutput;
use rs_render::renderer::Renderer;
use std::collections::{HashMap, VecDeque};
use std::path::Path;

struct State {
    is_cursor_visible: bool,
    camera_movement_speed: f32,
    camera_motion_speed: f32,
    #[cfg(not(target_os = "android"))]
    virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            is_cursor_visible: false,
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
    level: Option<rs_artifact::level::Level>,
    draw_objects: Vec<DrawObject>,
    camera: Camera,
    state: State,
    render_outputs: VecDeque<rs_render::command::RenderOutput>,
}

impl Engine {
    pub fn new<W>(
        window: &W,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
        artifact_reader: Option<ArtifactReader>,
    ) -> Result<Engine>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        let is_multiple_thread = true;
        let logger = Logger::new(LoggerConfiguration {
            is_write_to_file: true,
        });

        let renderer = Renderer::from_window(window, surface_width, surface_height, scale_factor);
        let mut renderer = match renderer {
            Ok(renderer) => renderer,
            Err(err) => return Err(crate::error::Error::RendererError(err)),
        };

        let mut draw_objects: Vec<DrawObject> = Vec::new();

        let mut resource_manager = ResourceManager::default();
        resource_manager.set_artifact_reader(artifact_reader);
        resource_manager.load_static_meshs();

        let mut shaders: HashMap<String, String> = HashMap::new();
        for shader_source_code in resource_manager.get_all_shader_source_codes() {
            shaders.insert(shader_source_code.url.to_string(), shader_source_code.code);
        }
        let mut render_thread_mode: ERenderThreadMode;
        if is_multiple_thread {
            render_thread_mode =
                ERenderThreadMode::Multiple(MultipleThreadRenderer::new(renderer, shaders));
        } else {
            renderer.load_shader(shaders);
            render_thread_mode = ERenderThreadMode::Single(renderer);
        }
        let camera = Camera::default(surface_width, surface_height);
        let mut level: Option<rs_artifact::level::Level> = None;
        if let Some(url) = Self::find_first_level(&mut resource_manager) {
            if let Ok(_level) = resource_manager.get_level(&url) {
                for node in &_level.nodes {
                    match node {
                        ENodeType::Node3D(node3d) => {
                            if let Some(mesh_url) = &node3d.mesh_url {
                                if let Ok(static_mesh) = resource_manager.get_static_mesh(mesh_url)
                                {
                                    let constants =
                                        rs_render::render_pipeline::phong_pipeline::Constants {
                                            model: glam::Mat4::IDENTITY,
                                            view: camera.get_view_matrix(),
                                            projection: camera.get_projection_matrix(),
                                        };
                                    let material = PhongMaterial {
                                        constants,
                                        diffuse_texture: None,
                                        specular_texture: None,
                                    };

                                    let draw_object =
                                        Self::create_draw_object_from_static_mesh_internal(
                                            &mut render_thread_mode,
                                            &mut resource_manager,
                                            &static_mesh.vertexes,
                                            &static_mesh.indexes,
                                            EMaterialType::Phong(material),
                                        );
                                    draw_objects.push(draw_object);
                                }
                            }
                        }
                    }
                }
                level = Some(_level);
            }
        }

        let engine = Engine {
            render_thread_mode,
            resource_manager,
            logger,
            level,
            draw_objects,
            camera,
            state: State::default(),
            render_outputs: VecDeque::new(),
        };

        Ok(engine)
    }

    fn find_first_level(resource_manager: &mut ResourceManager) -> Option<url::Url> {
        let Some(resource_map) = resource_manager.get_resource_map() else {
            return None;
        };
        for (k, v) in resource_map {
            if k.scheme() == "asset" {
                if let Some(host) = k.host() {
                    match host {
                        url::Host::Domain(host) => {
                            if host == "level" {
                                return Some(v.url);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        return None;
    }

    pub fn get_resource_map(&self) -> Option<HashMap<url::Url, ResourceInfo>> {
        self.resource_manager.get_resource_map()
    }

    pub fn redraw(&mut self, gui_render_output: EGUIRenderOutput) {
        loop {
            match &self.render_thread_mode {
                ERenderThreadMode::Single(_) => {
                    break;
                }
                ERenderThreadMode::Multiple(renderer) => {
                    let render_output = renderer.channel.from_b_try_recv();
                    match render_output {
                        Ok(render_output) => {
                            if let Some(render_output) = render_output {
                                self.render_outputs.push_back(render_output);
                            }
                        }
                        Err(err) => match err {
                            std::sync::mpsc::TryRecvError::Empty => {
                                break;
                            }
                            std::sync::mpsc::TryRecvError::Disconnected => {
                                panic!();
                            }
                        },
                    }
                }
            }
        }
        #[cfg(not(target_os = "android"))]
        for (virtual_key_code, element_state) in &self.state.virtual_key_code_states {
            DefaultCameraInputEventHandle::keyboard_input_handle(
                &mut self.camera,
                virtual_key_code,
                element_state,
                self.state.is_cursor_visible,
                self.state.camera_movement_speed,
            );
        }
        self.camera_did_update();

        for draw_object in &self.draw_objects {
            self.render_thread_mode
                .send_command(RenderCommand::DrawObject(draw_object.clone()));
        }

        self.render_thread_mode
            .send_command(RenderCommand::UiOutput(gui_render_output));
    }

    pub fn present(&mut self) {
        self.render_thread_mode.send_command(RenderCommand::Present);
    }

    pub fn resize(&mut self, surface_width: u32, surface_height: u32) {
        self.render_thread_mode.send_command(RenderCommand::Resize(
            rs_render::command::ResizeInfo {
                width: surface_width,
                height: surface_height,
            },
        ));
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
        match &mut self.render_thread_mode {
            ERenderThreadMode::Single(renderer) => {
                let result = renderer.set_new_window(window, surface_width, surface_height);
                match result {
                    Ok(_) => Ok(()),
                    Err(err) => return Err(crate::error::Error::RendererError(err)),
                }
            }
            ERenderThreadMode::Multiple(renderer) => {
                let result = renderer.renderer.lock().unwrap().set_new_window(
                    window,
                    surface_width,
                    surface_height,
                );
                match result {
                    Ok(_) => Ok(()),
                    Err(err) => return Err(crate::error::Error::RendererError(err)),
                }
            }
        }
    }

    fn create_draw_object_from_static_mesh_internal(
        render_thread_mode: &mut ERenderThreadMode,
        resource_manager: &mut ResourceManager,
        vertexes: &[rs_artifact::mesh_vertex::MeshVertex],
        indexes: &[u32],
        material_type: EMaterialType,
    ) -> DrawObject {
        let index_buffer_handle = resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some("StaticMesh::IndexBuffer".to_string()),
            contents: rs_foundation::cast_to_raw_buffer(&indexes).to_vec(),
            usage: wgpu::BufferUsages::INDEX,
        };
        let create_buffer = CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        render_thread_mode.send_command(message);

        let vertex_buffer_handle = resource_manager.next_buffer();
        let buffer_create_info = BufferCreateInfo {
            label: Some("StaticMesh::VertexBuffer".to_string()),
            contents: rs_foundation::cast_to_raw_buffer(&vertexes).to_vec(),
            usage: wgpu::BufferUsages::VERTEX,
        };
        let create_buffer = CreateBuffer {
            handle: *vertex_buffer_handle,
            buffer_create_info,
        };
        let message = RenderCommand::CreateBuffer(create_buffer);
        render_thread_mode.send_command(message);

        let draw_object = DrawObject {
            vertex_buffers: vec![*vertex_buffer_handle],
            vertex_count: vertexes.len() as u32,
            index_buffer: Some(*index_buffer_handle),
            index_count: Some(indexes.len() as u32),
            material_type,
        };
        draw_object
    }

    pub fn create_draw_object_from_static_mesh(
        &mut self,
        vertexes: &[rs_artifact::mesh_vertex::MeshVertex],
        indexes: &[u32],
        material_type: EMaterialType,
    ) -> DrawObject {
        Self::create_draw_object_from_static_mesh_internal(
            &mut self.render_thread_mode,
            &mut self.resource_manager,
            vertexes,
            indexes,
            material_type,
        )
    }

    pub fn draw(&mut self, draw_object: DrawObject) {
        self.render_thread_mode
            .send_command(RenderCommand::DrawObject(draw_object));
    }

    #[cfg(not(target_os = "android"))]
    pub fn process_device_event(&mut self, device_event: winit::event::DeviceEvent) {
        match device_event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                DefaultCameraInputEventHandle::mouse_motion_handle(
                    &mut self.camera,
                    delta,
                    self.state.is_cursor_visible,
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

    fn camera_did_update(&mut self) {
        for draw_objects in &mut self.draw_objects {
            match &mut draw_objects.material_type {
                rs_render::command::EMaterialType::Phong(material) => {
                    material.constants.projection = self.camera.get_projection_matrix();
                    material.constants.view = self.camera.get_view_matrix();
                }
                rs_render::command::EMaterialType::PBR(_) => {}
            }
        }
    }

    pub fn get_mut_resource_manager(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }

    pub fn create_texture_from_path(
        &mut self,
        path: &Path,
        url: url::Url,
    ) -> Option<crate::handle::TextureHandle> {
        let image = match image::open(path) {
            Ok(dynamic_image) => match dynamic_image {
                image::DynamicImage::ImageRgba8(image) => image,
                x => x.to_rgba8(),
            },
            Err(err) => {
                log::warn!("{}", err);
                return None;
            }
        };
        let handle = self.resource_manager.next_texture(url.clone());
        let create_texture = CreateTexture {
            handle: *handle,
            texture_descriptor_create_info: TextureDescriptorCreateInfo::d2(
                Some(String::from(format!("{:?}", url))),
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
        return Some(handle);
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        match &self.render_thread_mode {
            ERenderThreadMode::Single(_) => {}
            ERenderThreadMode::Multiple(renderer) => {
                renderer.channel.send_stop_signal_and_wait();
            }
        }
        self.logger.flush();
    }
}
