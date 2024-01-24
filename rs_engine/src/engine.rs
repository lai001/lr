use crate::camera::Camera;
#[cfg(not(target_os = "android"))]
use crate::camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle};
use crate::error::Result;
use crate::thread_pool;
use crate::{
    logger::{Logger, LoggerConfiguration},
    resource_manager::ResourceManager,
};
use rs_artifact::artifact::ArtifactReader;
use rs_artifact::level::ENodeType;
use rs_artifact::resource_info::ResourceInfo;
use rs_foundation::channel::SingleConsumeChnnel;
use rs_render::command::{
    BufferCreateInfo, CreateBuffer, DrawObject, EMaterialType, PhongMaterial, RenderCommand,
    RenderOutput,
};
use rs_render::renderer::Renderer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct State {
    is_cursor_visible: bool,
    camera_movement_speed: f32,
    camera_motion_speed: f32,
    #[cfg(not(target_os = "android"))]
    virtual_key_code_states: HashMap<winit::event::VirtualKeyCode, winit::event::ElementState>,
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
    renderer: Arc<Mutex<Renderer>>,
    channel: Arc<SingleConsumeChnnel<RenderCommand, Option<RenderOutput>>>,
    resource_manager: ResourceManager,
    logger: Logger,
    gui_context: egui::Context,
    level: Option<rs_artifact::level::Level>,
    draw_objects: Vec<DrawObject>,
    camera: Camera,
    state: State,
}

impl Engine {
    pub fn new<W>(
        window: &W,
        surface_width: u32,
        surface_height: u32,
        scale_factor: f32,
        gui_context: egui::Context,
        artifact_reader: Option<ArtifactReader>,
    ) -> Result<Engine>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let logger = Logger::new(LoggerConfiguration {
            is_write_to_file: true,
        });

        let renderer = Renderer::from_window(
            window,
            gui_context.clone(),
            surface_width,
            surface_height,
            scale_factor,
        );
        let renderer = match renderer {
            Ok(renderer) => renderer,
            Err(err) => return Err(crate::error::Error::RendererError(err)),
        };
        let renderer = Arc::new(Mutex::new(renderer));
        let mut draw_objects: Vec<DrawObject> = Vec::new();

        let mut resource_manager = ResourceManager::default();
        resource_manager.set_artifact_reader(artifact_reader);
        resource_manager.load_static_meshs();

        let mut shaders: HashMap<String, String> = HashMap::new();
        for shader_source_code in resource_manager.get_all_shader_source_codes() {
            shaders.insert(shader_source_code.url.to_string(), shader_source_code.code);
        }

        let channel = Self::spawn_render_thread(renderer.clone(), shaders);
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
                                            &channel,
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
            renderer,
            resource_manager,
            logger,
            gui_context,
            channel,
            level,
            draw_objects,
            camera,
            state: State::default(),
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

    fn spawn_render_thread(
        renderer: Arc<Mutex<Renderer>>,
        shaders: HashMap<String, String>,
    ) -> Arc<SingleConsumeChnnel<RenderCommand, Option<RenderOutput>>> {
        let channel =
            SingleConsumeChnnel::<RenderCommand, Option<RenderOutput>>::shared(Some(2), None);
        thread_pool::ThreadPool::render().spawn({
            let renderer = renderer.clone();
            let shaders = shaders.clone();
            let channel = channel.clone();

            move || {
                {
                    let mut renderer = renderer.lock().unwrap();
                    renderer.load_shader(shaders);
                }

                channel.from_a_block_current_thread(|command| {
                    let mut renderer = renderer.lock().unwrap();
                    let output = renderer.send_command(command);
                    channel.to_a(output);
                });
            }
        });
        return channel;
    }

    pub fn redraw(&mut self, full_output: egui::FullOutput) {
        loop {
            match self.channel.from_b_try_recv() {
                Ok(_) => {}
                Err(_) => break,
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
            self.channel
                .to_b(RenderCommand::DrawObject(draw_object.clone()));
        }
        self.channel.to_b(RenderCommand::UiOutput(full_output));
        self.channel.to_b(RenderCommand::Present);
    }

    pub fn resize(&mut self, surface_width: u32, surface_height: u32) {
        self.channel
            .to_b(RenderCommand::Resize(rs_render::command::ResizeInfo {
                width: surface_width,
                height: surface_height,
            }));
    }

    pub fn set_new_window<W>(
        &mut self,
        window: &W,
        surface_width: u32,
        surface_height: u32,
    ) -> Result<()>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let result =
            self.renderer
                .lock()
                .unwrap()
                .set_new_window(window, surface_width, surface_height);
        match result {
            Ok(_) => Ok(()),
            Err(err) => return Err(crate::error::Error::RendererError(err)),
        }
    }

    pub fn get_gui_context(&self) -> egui::Context {
        self.gui_context.clone()
    }

    fn create_draw_object_from_static_mesh_internal(
        channel: &Arc<SingleConsumeChnnel<RenderCommand, Option<RenderOutput>>>,
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
        channel.to_b(message);

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
        channel.to_b(message);

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
            &self.channel,
            &mut self.resource_manager,
            vertexes,
            indexes,
            material_type,
        )
    }

    pub fn draw(&mut self, draw_object: DrawObject) {
        self.channel.to_b(RenderCommand::DrawObject(draw_object));
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
    pub fn process_keyboard_input(&mut self, input: winit::event::KeyboardInput) {
        let Some(virtual_keycode) = input.virtual_keycode else {
            return;
        };
        self.state
            .virtual_key_code_states
            .insert(virtual_keycode, input.state);
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
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.channel.send_stop_signal_and_wait();
        self.logger.flush();
    }
}
