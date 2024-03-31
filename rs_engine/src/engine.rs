use crate::camera::Camera;
#[cfg(not(target_os = "android"))]
use crate::camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle};
use crate::error::Result;
use crate::render_thread_mode::ERenderThreadMode;
use crate::{logger::Logger, resource_manager::ResourceManager};
use rs_artifact::artifact::ArtifactReader;
use rs_artifact::level::ENodeType;
use rs_artifact::resource_info::ResourceInfo;
use rs_core_minimal::settings::Settings;
use rs_render::bake_info::BakeInfo;
use rs_render::command::{
    BufferCreateInfo, CreateBuffer, CreateIBLBake, CreateTexture, CreateVirtualTexture, DrawObject,
    EMaterialType, InitTextureData, PhongMaterial, RenderCommand, TextureDescriptorCreateInfo,
};
use rs_render::egui_render::EGUIRenderOutput;
use rs_render::renderer::Renderer;
use rs_render::virtual_texture_source::TVirtualTextureSource;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

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
    draw_object_id: u32,
    camera: Camera,
    state: State,
    settings: Settings,
}

impl Engine {
    pub fn new<W>(
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
        let mut resource_manager = ResourceManager::default();
        resource_manager.set_artifact_reader(artifact_reader);
        resource_manager.load_static_meshs();

        for shader_source_code in resource_manager.get_all_shader_source_codes() {
            shaders.insert(shader_source_code.name, shader_source_code.code);
        }

        let renderer = Renderer::from_window(
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
        let camera = Camera::default(surface_width, surface_height);
        let mut level: Option<rs_artifact::level::Level> = None;
        let mut draw_object_id: u32 = 0;
        (|| {
            let Some(url) = Self::find_first_level(&mut resource_manager) else {
                return;
            };
            let Ok(_level) = resource_manager.get_level(&url) else {
                return;
            };
            for node in &_level.nodes {
                match node {
                    ENodeType::Node3D(node3d) => {
                        let Some(mesh_url) = &node3d.mesh_url else {
                            continue;
                        };
                        let Ok(static_mesh) = resource_manager.get_static_mesh(mesh_url) else {
                            continue;
                        };

                        let constants = rs_render::render_pipeline::phong_pipeline::Constants {
                            model: glam::Mat4::IDENTITY,
                            view: camera.get_view_matrix(),
                            projection: camera.get_projection_matrix(),
                        };
                        let material = PhongMaterial {
                            constants,
                            diffuse_texture: None,
                            specular_texture: None,
                        };

                        let draw_object = Self::create_draw_object_from_static_mesh_internal(
                            &mut render_thread_mode,
                            &mut resource_manager,
                            &static_mesh.vertexes,
                            &static_mesh.indexes,
                            EMaterialType::Phong(material),
                            draw_object_id,
                        );
                        draw_object_id += 1;
                        draw_objects.push(draw_object);
                    }
                }
            }
            level = Some(_level);
        })();

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
        };

        Ok(engine)
    }

    fn find_first_level(resource_manager: &mut ResourceManager) -> Option<url::Url> {
        let Ok(resource_map) = resource_manager.get_resource_map() else {
            return None;
        };
        for (k, v) in resource_map {
            if k.scheme() != "asset" {
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
        self.render_thread_mode
            .set_new_window(window, surface_width, surface_height)
    }

    fn create_draw_object_from_static_mesh_internal(
        render_thread_mode: &mut ERenderThreadMode,
        resource_manager: &mut ResourceManager,
        vertexes: &[rs_artifact::mesh_vertex::MeshVertex],
        indexes: &[u32],
        material_type: EMaterialType,
        id: u32,
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
            id,
        };
        draw_object
    }

    pub fn create_draw_object_from_static_mesh(
        &mut self,
        vertexes: &[rs_artifact::mesh_vertex::MeshVertex],
        indexes: &[u32],
        material_type: EMaterialType,
    ) -> DrawObject {
        let draw_object = Self::create_draw_object_from_static_mesh_internal(
            &mut self.render_thread_mode,
            &mut self.resource_manager,
            vertexes,
            indexes,
            material_type,
            self.draw_object_id,
        );
        self.draw_object_id += 1;
        draw_object
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
        let handle = self.resource_manager.next_virtual_texture(url.clone());
        let command = CreateVirtualTexture {
            handle: *handle,
            source: Arc::new(Mutex::new(source)),
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
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.logger.flush();
    }
}
