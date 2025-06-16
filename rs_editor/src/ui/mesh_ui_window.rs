use super::{misc::update_window_with_input_mode, ui_window::UIWindow};
use crate::{editor_context::EWindowType, windows_manager::WindowsManager};
use anyhow::anyhow;
use egui_winit::State;
use rs_artifact::skin_mesh::SkinMeshVertex;
use rs_engine::{
    camera::Camera,
    camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle},
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    handle::BufferHandle,
    input_mode::EInputMode,
    resource_manager::ResourceManager,
};
use rs_mesh_optimization::optimization::MeshoptMesh;
use rs_metis::{cluster::ClusterCollection, vertex_position::VertexPosition};
use rs_render::{
    command::{
        BufferCreateInfo, CreateBuffer, DrawObject, EBindingResource, PresentInfo, RenderCommand,
        UpdateBuffer,
    },
    constants::MeshViewConstants,
    renderer::{EBuiltinPipelineType, EPipelineType},
    vertex_data_type::mesh_vertex::MeshVertex3,
};
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};
use winit::event::{MouseButton, MouseScrollDelta, WindowEvent};

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
    input_mode: EInputMode,
    window_id: isize,
}

impl UIWindow for MeshUIWindow {
    fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent) {
        match device_event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                DefaultCameraInputEventHandle::mouse_motion_handle(
                    &mut self.camera,
                    *delta,
                    self.input_mode,
                    self.camera_motion_speed,
                );
            }
            _ => {}
        }
    }

    fn on_window_event(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
        window_manager: &mut WindowsManager,
        is_request_close: &mut bool,
    ) {
        let _ = window_manager;
        let _ = is_request_close;
        let _ = window;
        let _ = event_loop_window_target;
        match event {
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, up) => {
                    self.camera_movement_speed += up * 0.005;
                    self.camera_movement_speed = self.camera_movement_speed.max(0.0);
                }
                MouseScrollDelta::PixelDelta(_) => todo!(),
            },
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Right {
                    match state {
                        winit::event::ElementState::Pressed => {
                            self.input_mode = EInputMode::Game;
                        }
                        winit::event::ElementState::Released => {
                            self.input_mode = EInputMode::UI;
                        }
                    }
                    update_window_with_input_mode(window, self.input_mode);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let winit::keyboard::PhysicalKey::Code(virtual_keycode) = event.physical_key else {
                    return;
                };
                self.virtual_key_code_states
                    .insert(virtual_keycode, event.state);
            }
            WindowEvent::RedrawRequested => {
                engine.window_redraw_requested_begin(window_id);

                for (virtual_key_code, element_state) in &self.virtual_key_code_states {
                    DefaultCameraInputEventHandle::keyboard_input_handle(
                        &mut self.camera,
                        virtual_key_code,
                        element_state,
                        self.input_mode,
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

                engine.send_render_command(RenderCommand::Present(PresentInfo::new(
                    rs_render::command::ERenderTargetType::SurfaceTexture(window_id),
                    present_draw_objects,
                )));
                self.frame_sync.sync(60.0);
                engine.window_redraw_requested_end(window_id);
                window.request_redraw();
            }
            _ => {}
        }
    }

    fn get_window_id(&self) -> isize {
        self.window_id
    }
}

impl MeshUIWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
    ) -> anyhow::Result<MeshUIWindow> {
        let window_context =
            window_manager.spwan_new_window(EWindowType::Mesh, event_loop_window_target, None)?;
        let window = &*window_context.window.borrow();
        let window_id = window_context.get_id();

        engine
            .set_new_window(
                window_context.get_id(),
                window,
                window_context.get_width(),
                window_context.get_height(),
                window.scale_factor() as f32,
            )
            .map_err(|err| anyhow!("{err}"))?;
        let viewport_id = egui::ViewportId::from_hash_of(window_context.get_id());

        let mut egui_winit_state = egui_winit::State::new(
            context,
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        egui_winit_state.egui_input_mut().viewport_id = viewport_id;
        egui_winit_state.egui_input_mut().viewports =
            std::iter::once((viewport_id, Default::default())).collect();

        let mut camera = Camera::default(window_context.get_width(), window_context.get_height());
        camera.set_world_location(glam::Vec3 {
            x: 0.0,
            y: 15.0,
            z: -50.0,
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

        let grid_draw_object = engine.create_grid_draw_object(global_constants_handle.clone());
        let input_mode = EInputMode::UI;
        update_window_with_input_mode(window, input_mode);
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
            input_mode,
            window_id,
        })
    }

    fn _make_lods(
        engine: &mut Engine,
        skin_mesh_vertices: &[SkinMeshVertex],
        indices: &[u32],
        global_constants_handle: BufferHandle,
    ) -> Vec<MeshViewDrawObject> {
        let mesh = MeshoptMesh {
            vertices: skin_mesh_vertices
                .iter()
                .map(|item| {
                    //
                    meshopt::Vertex {
                        p: item.position.into(),
                        n: item.normal.into(),
                        t: item.tex_coord.into(),
                    }
                })
                .collect(),
            indices: indices.to_vec(),
        };
        let lods =
            rs_mesh_optimization::optimization::simplify(&mesh, NonZeroUsize::new(8).unwrap());
        let mut mesh_draw_objects = vec![];
        for (i, lod_indices) in lods.iter().enumerate() {
            let location = glam::vec3(i as f32 * 15.0, 0.0, 0.0);
            let mesh_draw_object = Self::_make_mesh_draw_object(
                engine,
                skin_mesh_vertices,
                lod_indices,
                global_constants_handle.clone(),
                location,
            );
            mesh_draw_objects.push(mesh_draw_object);
        }
        return mesh_draw_objects;
    }

    fn _make_mesh_draw_object(
        engine: &mut Engine,
        mesh_vertices: &[SkinMeshVertex],
        indices: &[u32],
        global_constants_handle: BufferHandle,
        location: glam::Vec3,
    ) -> MeshViewDrawObject {
        let resource_manager = ResourceManager::default();

        let mut vertices: Vec<MeshVertex3> = vec![MeshVertex3::default(); mesh_vertices.len()];

        for (mesh_vertices, skin_mesh_vertices) in
            vertices.chunks_mut(3).zip(mesh_vertices.chunks(3))
        {
            let color = rs_core_minimal::color::random_color4();
            for item in mesh_vertices.iter_mut().zip(skin_mesh_vertices) {
                *item.0 = MeshVertex3 {
                    position: item.1.position,
                    vertex_color: color,
                };
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
        let mut mesh_view_constants = MeshViewConstants::default();
        mesh_view_constants.model = glam::Mat4::from_translation(location);

        engine.send_render_command(RenderCommand::CreateBuffer(CreateBuffer {
            handle: *constants_handle,
            buffer_create_info: BufferCreateInfo {
                label: None,
                contents: rs_foundation::cast_any_as_u8_slice(&mesh_view_constants).to_vec(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
            },
        }));

        let draw_object = DrawObject::new(
            0,
            vec![*vertex_buffer_handle],
            vertices.len() as u32,
            EPipelineType::Builtin(EBuiltinPipelineType::MeshView),
            Some(*index_buffer_handle),
            Some(indices.len() as u32),
            vec![vec![
                EBindingResource::Constants(*global_constants_handle),
                EBindingResource::Constants(*constants_handle),
            ]],
        );

        MeshViewDrawObject {
            draw_object,
            constants_handle,
            mesh_view_constants,
        }
    }

    fn make_cluster_lods(
        engine: &mut Engine,
        mesh_vertices: &[rs_artifact::mesh_vertex::MeshVertex],
        indices: &[u32],
        global_constants_handle: BufferHandle,
    ) -> Vec<MeshViewDrawObject> {
        let mut mesh_draw_objects = vec![];
        let mut vertices: Vec<VertexPosition> = Vec::with_capacity(mesh_vertices.len());
        for item in mesh_vertices {
            vertices.push(VertexPosition::new(item.position));
        }
        let vertices = Arc::new(vertices);
        let cluster_collection = match ClusterCollection::parallel_from_indexed_vertices2(
            Arc::new(indices.to_vec()),
            vertices,
        ) {
            Ok(cluster_collection) => cluster_collection,
            Err(err) => {
                log::warn!("{}", err);
                return mesh_draw_objects;
            }
        };
        let resource_manager = ResourceManager::default();
        let cluster_collections = cluster_collection.plat();

        for (i, cluster_collection) in cluster_collections.iter().enumerate() {
            let location = glam::vec3(i as f32 * 15.0, 0.0, 0.0);

            let mesh_vertices_num = cluster_collection.iter().map(|x| x.indices.len()).sum();
            let mut vertices: Vec<MeshVertex3> = vec![MeshVertex3::default(); mesh_vertices_num];
            for cluster in cluster_collection {
                let vertex_color = rs_core_minimal::color::random_color4();
                for vertex_index in &cluster.indices {
                    let vertex = &mesh_vertices[*vertex_index as usize];
                    let vertex = MeshVertex3 {
                        position: vertex.position,
                        vertex_color,
                    };
                    vertices.push(vertex);
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

            let constants_handle = resource_manager.next_buffer();
            let mut mesh_view_constants = MeshViewConstants::default();
            mesh_view_constants.model = glam::Mat4::from_translation(location);

            engine.send_render_command(RenderCommand::CreateBuffer(CreateBuffer {
                handle: *constants_handle,
                buffer_create_info: BufferCreateInfo {
                    label: None,
                    contents: rs_foundation::cast_any_as_u8_slice(&mesh_view_constants).to_vec(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
                },
            }));

            let draw_object = DrawObject::new(
                0,
                vec![*vertex_buffer_handle],
                vertices.len() as u32,
                EPipelineType::Builtin(EBuiltinPipelineType::MeshView),
                None,
                None,
                vec![vec![
                    EBindingResource::Constants(*global_constants_handle),
                    EBindingResource::Constants(*constants_handle),
                ]],
            );
            mesh_draw_objects.push(MeshViewDrawObject {
                draw_object,
                constants_handle,
                mesh_view_constants,
            });
        }

        return mesh_draw_objects;
    }

    pub fn update(
        &mut self,
        engine: &mut Engine,
        skin_mesh_vertices: &[SkinMeshVertex],
        indices: &[u32],
    ) {
        // self.draw_objects = Self::make_lods(
        //     engine,
        //     skin_mesh_vertices,
        //     indices,
        //     self.global_constants_handle.clone(),
        // );

        self.draw_objects = Self::make_cluster_lods(
            engine,
            &skin_mesh_vertices
                .iter()
                .map(|x| x.to_mesh_vertex())
                .collect::<Vec<rs_artifact::mesh_vertex::MeshVertex>>(),
            indices,
            self.global_constants_handle.clone(),
        );
    }

    pub fn update2(
        &mut self,
        engine: &mut Engine,
        mesh_vertices: &[rs_artifact::mesh_vertex::MeshVertex],
        indices: &[u32],
    ) {
        // self.draw_objects = Self::make_lods(
        //     engine,
        //     skin_mesh_vertices,
        //     indices,
        //     self.global_constants_handle.clone(),
        // );

        self.draw_objects = Self::make_cluster_lods(
            engine,
            mesh_vertices,
            indices,
            self.global_constants_handle.clone(),
        );
    }
}
