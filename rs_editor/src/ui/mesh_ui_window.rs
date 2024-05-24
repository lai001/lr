use crate::{custom_event::ECustomEventType, editor::WindowsManager, editor_context::EWindowType};
use anyhow::anyhow;
use egui_winit::State;
use rs_artifact::skin_mesh::SkinMeshVertex;
use rs_core_minimal::file_manager::get_gpmetis_program_path;
use rs_engine::{
    camera::Camera,
    camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle},
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    handle::BufferHandle,
    input_mode::EInputMode,
    resource_manager::ResourceManager,
};
use rs_render::{
    command::{
        BufferCreateInfo, CreateBuffer, DrawObject, EBindingResource, PresentInfo, RenderCommand,
        UpdateBuffer,
    },
    constants::MeshViewConstants,
    renderer::MESH_VIEW_RENDER_PIPELINE,
    vertex_data_type::mesh_vertex::MeshVertex3,
};
use std::collections::HashMap;
use winit::event::{MouseScrollDelta, WindowEvent};

struct MeshViewDrawObject {
    draw_object: rs_render::command::DrawObject,
    constants_handle: BufferHandle,
    mesh_view_constants: MeshViewConstants,
}

pub struct MeshUIWindow {
    pub egui_winit_state: State,
    draw_objects: Vec<MeshViewDrawObject>,
    camera: Camera,
    frame_sync: FrameSync,
    virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
    global_constants: rs_render::global_uniform::Constants,
    global_constants_handle: rs_engine::handle::BufferHandle,
    grid_draw_object: DrawObject,
    camera_movement_speed: f32,
    camera_motion_speed: f32,
}

impl MeshUIWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
        engine: &mut Engine,
    ) -> anyhow::Result<MeshUIWindow> {
        let window_context =
            window_manager.spwan_new_window(EWindowType::Mesh, event_loop_window_target)?;
        let window = &*window_context.window.borrow();

        engine
            .set_new_window(
                window_context.get_id(),
                window,
                window_context.get_width(),
                window_context.get_height(),
            )
            .map_err(|err| anyhow!("{err}"))?;
        let viewport_id = egui::ViewportId::from_hash_of(window_context.get_id());

        let mut egui_winit_state = egui_winit::State::new(
            context,
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
        );

        egui_winit_state.egui_input_mut().viewport_id = viewport_id;
        egui_winit_state.egui_input_mut().viewports =
            std::iter::once((viewport_id, Default::default())).collect();

        let mut camera = Camera::default(window_context.get_width(), window_context.get_height());
        camera.set_world_location(glam::Vec3 {
            x: 0.0,
            y: 3.0,
            z: 3.0,
        });
        let frame_sync = FrameSync::new(EOptions::FPS(60.0));

        let resource_manager = ResourceManager::default();
        let global_constants_handle = resource_manager.next_buffer();
        let mut global_constants = rs_render::global_uniform::Constants::default();

        global_constants.view_projection = camera.get_view_projection_matrix();
        global_constants.view = camera.get_view_matrix();
        global_constants.projection = camera.get_projection_matrix();
        global_constants.view_position = camera.get_world_location();

        let command = RenderCommand::CreateBuffer(CreateBuffer {
            handle: *global_constants_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some("Global.Constants".to_string()),
                contents: rs_foundation::cast_to_raw_buffer(&vec![global_constants]).to_vec(),
                usage: wgpu::BufferUsages::all(),
            },
        });
        engine.send_render_command(command);

        let grid_draw_object = engine.create_grid_draw_object(0, global_constants_handle.clone());

        Ok(MeshUIWindow {
            egui_winit_state,
            draw_objects: vec![],
            camera,
            frame_sync,
            virtual_key_code_states: HashMap::new(),
            global_constants,
            global_constants_handle,
            grid_draw_object,
            camera_movement_speed: 0.01,
            camera_motion_speed: 0.1,
        })
    }

    pub fn device_event_process(&mut self, device_event: &winit::event::DeviceEvent) {
        match device_event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                let input_mode = EInputMode::Game;
                DefaultCameraInputEventHandle::mouse_motion_handle(
                    &mut self.camera,
                    *delta,
                    input_mode,
                    self.camera_motion_speed,
                );
            }
            _ => {}
        }
    }

    pub fn window_event_process(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
        engine: &mut Engine,
        window_manager: &mut WindowsManager,
    ) {
        let _ = window;
        let _ = event_loop_window_target;
        match event {
            WindowEvent::Resized(size) => {
                engine.resize(window_id, size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                window_manager.remove_window(EWindowType::Mesh);
                engine.remove_window(window_id);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, up) => {
                    self.camera_movement_speed += up * 0.005;
                    self.camera_movement_speed = self.camera_movement_speed.max(0.0);
                }
                MouseScrollDelta::PixelDelta(_) => todo!(),
            },
            WindowEvent::KeyboardInput { event, .. } => {
                let winit::keyboard::PhysicalKey::Code(virtual_keycode) = event.physical_key else {
                    return;
                };
                self.virtual_key_code_states
                    .insert(virtual_keycode, event.state);
            }
            WindowEvent::RedrawRequested => {
                engine.recv_output_hook();

                for (virtual_key_code, element_state) in &self.virtual_key_code_states {
                    let input_mode = EInputMode::Game;
                    DefaultCameraInputEventHandle::keyboard_input_handle(
                        &mut self.camera,
                        virtual_key_code,
                        element_state,
                        input_mode,
                        self.camera_movement_speed,
                    );
                }

                self.global_constants.view_projection = self.camera.get_view_projection_matrix();
                self.global_constants.view = self.camera.get_view_matrix();
                self.global_constants.projection = self.camera.get_projection_matrix();
                self.global_constants.view_position = self.camera.get_world_location();
                let command = RenderCommand::UpdateBuffer(UpdateBuffer {
                    handle: *self.global_constants_handle,
                    data: rs_foundation::cast_to_raw_buffer(&vec![self.global_constants]).to_vec(),
                });
                engine.send_render_command(command);

                for draw_object in self.draw_objects.iter_mut() {
                    engine.send_render_command(RenderCommand::UpdateBuffer(UpdateBuffer {
                        handle: *draw_object.constants_handle,
                        data: rs_foundation::cast_any_as_u8_slice(&draw_object.mesh_view_constants)
                            .to_vec(),
                    }));
                }

                let mut present_draw_objects: Vec<DrawObject> = vec![];
                present_draw_objects.extend(
                    self.draw_objects
                        .iter()
                        .map(|x| x.draw_object.clone())
                        .collect::<Vec<DrawObject>>(),
                );
                present_draw_objects.push(self.grid_draw_object.clone());

                engine.send_render_command(RenderCommand::Present(PresentInfo {
                    window_id,
                    draw_objects: present_draw_objects,
                    virtual_texture_pass: None,
                }));
                let wait = self
                    .frame_sync
                    .tick()
                    .unwrap_or(std::time::Duration::from_secs_f32(1.0 / 60.0));
                std::thread::sleep(wait);

                window.request_redraw();
            }
            _ => {}
        }
    }

    pub fn update(
        &mut self,
        engine: &mut Engine,
        skin_mesh_vertices: &[SkinMeshVertex],
        indices: &[u32],
    ) {
        let vertices = skin_mesh_vertices
            .iter()
            .map(|x| x.position)
            .collect::<Vec<glam::Vec3>>();
        let num_parts = 300;
        let mesh_clusters = rs_metis::metis::Metis::partition(
            &indices,
            vertices.as_slice(),
            num_parts as u32,
            get_gpmetis_program_path(),
        )
        .unwrap();
        let resource_manager = ResourceManager::default();
        self.draw_objects.clear();

        let mut vertices: Vec<MeshVertex3> = vec![];
        for mesh_cluster in mesh_clusters {
            let color = Self::random_color();
            for index in mesh_cluster {
                for offset in 0..=2 {
                    let vertex_index = indices[index + offset];
                    let vertex = &skin_mesh_vertices[vertex_index as usize];
                    let mesh_vertex3 = MeshVertex3 {
                        position: vertex.position,
                        vertex_color: color,
                    };
                    vertices.push(mesh_vertex3);
                }
            }
        }

        let vertex_buffer_handle = resource_manager.next_buffer();
        engine.send_render_command(RenderCommand::CreateBuffer(CreateBuffer {
            handle: *vertex_buffer_handle,
            buffer_create_info: BufferCreateInfo {
                label: None,
                contents: rs_foundation::cast_to_raw_buffer(&vertices).to_vec(),
                usage: wgpu::BufferUsages::VERTEX,
            },
        }));

        let indices: Vec<u32> = (0..vertices.len()).map(|x| x as u32).collect();
        let index_buffer_handle = resource_manager.next_buffer();
        engine.send_render_command(RenderCommand::CreateBuffer(CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info: BufferCreateInfo {
                label: None,
                contents: rs_foundation::cast_to_raw_buffer(&indices).to_vec(),
                usage: wgpu::BufferUsages::INDEX,
            },
        }));

        let constants_handle = resource_manager.next_buffer();
        let mesh_view_constants = MeshViewConstants::default();

        engine.send_render_command(RenderCommand::CreateBuffer(CreateBuffer {
            handle: *constants_handle,
            buffer_create_info: BufferCreateInfo {
                label: None,
                contents: rs_foundation::cast_any_as_u8_slice(&mesh_view_constants).to_vec(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
            },
        }));

        let draw_object = DrawObject {
            id: 0,
            vertex_buffers: vec![*vertex_buffer_handle],
            vertex_count: vertices.len() as u32,
            index_buffer: Some(*index_buffer_handle),
            index_count: Some(indices.len() as u32),
            binding_resources: vec![vec![
                EBindingResource::Constants(*self.global_constants_handle),
                EBindingResource::Constants(*constants_handle),
            ]],
            virtual_pass_set: None,
            render_pipeline: MESH_VIEW_RENDER_PIPELINE.to_string(),
        };
        self.draw_objects.push(MeshViewDrawObject {
            draw_object,
            constants_handle,
            mesh_view_constants,
        });
    }

    fn random_color() -> glam::Vec3 {
        let x: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
        let y: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
        let z: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..1.0);
        glam::vec3(x, y, z)
    }
}
