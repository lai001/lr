use super::{misc::update_window_with_input_mode, ui_window::UIWindow};
use crate::{editor_context::EWindowType, windows_manager::WindowsManager};
use anyhow::anyhow;
use egui_winit::State;
use rs_core_minimal::primitive_data::PrimitiveData;
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
        BufferCreateInfo, CreateBuffer, DrawObject, EBindingResource, EDrawCallType,
        MultiDrawIndirect, PresentInfo, RenderCommand, UpdateBuffer,
    },
    constants::MeshViewConstants,
    renderer::{EBuiltinPipelineType, EPipelineType},
    vertex_data_type::mesh_vertex::MeshVertex4,
};
use std::collections::HashMap;
use wgpu::util::DrawIndexedIndirectArgs;
use winit::event::{MouseButton, MouseScrollDelta, WindowEvent};

struct MeshViewDrawObject {
    draw_object: rs_render::command::DrawObject,
    constants_handle: BufferHandle,
    mesh_view_constants_array: Vec<MeshViewConstants>,
}

pub struct MultipleDrawUiWindow {
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
}

impl UIWindow for MultipleDrawUiWindow {
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
                engine.tick();
                for draw_object in self.draw_objects.iter_mut() {
                    for (_, mesh_view_constants) in
                        draw_object.mesh_view_constants_array.iter_mut().enumerate()
                    {
                        let x: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..0.1);
                        let r = x * engine.get_game_time().sin();
                        mesh_view_constants.model = glam::Mat4::from_rotation_x(r)
                            * glam::Mat4::from_rotation_y(r)
                            * glam::Mat4::from_rotation_z(r)
                            * mesh_view_constants.model;
                    }
                    engine.send_render_command(RenderCommand::UpdateBuffer(UpdateBuffer {
                        handle: *draw_object.constants_handle,
                        data: rs_foundation::cast_to_raw_buffer(
                            &draw_object.mesh_view_constants_array,
                        )
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
}

impl MultipleDrawUiWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
    ) -> anyhow::Result<MultipleDrawUiWindow> {
        let window_context =
            window_manager.spwan_new_window(EWindowType::MultipleDraw, event_loop_window_target)?;
        let window = &*window_context.window.borrow();

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

        let grid_draw_object = engine.create_grid_draw_object(global_constants_handle.clone());
        let input_mode = EInputMode::UI;
        update_window_with_input_mode(window, input_mode);
        Ok(MultipleDrawUiWindow {
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
        })
    }

    pub fn update(&mut self, engine: &mut Engine) {
        let resource_manager = ResourceManager::default();

        const REPEAT_SIZE: usize = 5000;
        let quad = PrimitiveData::quad();

        let vertices = (0..REPEAT_SIZE)
            .flat_map(|id| {
                let vertex_color = rs_core_minimal::color::random_color3();
                quad.into_iter()
                    .map(|(_, vertex_position, ..)| MeshVertex4 {
                        position: *vertex_position,
                        vertex_color,
                        draw_id: id as u32,
                    })
                    .collect::<Vec<MeshVertex4>>()
            })
            .collect::<Vec<MeshVertex4>>();

        let indices = quad.indices.repeat(REPEAT_SIZE);
        let draw_indexed_indirect_args_array = (0..REPEAT_SIZE)
            .map(|x| DrawIndexedIndirectArgs {
                index_count: quad.indices.len() as u32,
                instance_count: 1,
                first_index: quad.indices.len() as u32 * x as u32,
                base_vertex: quad.vertex_positions.len() as i32 * x as i32,
                first_instance: 0 as u32,
            })
            .collect::<Vec<DrawIndexedIndirectArgs>>();

        let constants_handle = resource_manager.next_buffer();
        let mut mesh_view_constants_array: Vec<MeshViewConstants> =
            vec![MeshViewConstants::default(); REPEAT_SIZE];

        for (_, mesh_view_constants) in mesh_view_constants_array.iter_mut().enumerate() {
            let offset: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), -500.0..500.0);
            let x: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), -1.0..1.0) * offset;
            let y: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), -1.0..1.0) * offset;
            let z: f32 = rand::Rng::gen_range(&mut rand::thread_rng(), -1.0..1.0) * offset;
            let translation = glam::Mat4::from_translation(glam::vec3(x, y, z));
            mesh_view_constants.model = translation;
        }

        engine.send_render_command(RenderCommand::CreateBuffer(CreateBuffer {
            handle: *constants_handle,
            buffer_create_info: BufferCreateInfo {
                label: None,
                contents: rs_foundation::cast_to_raw_buffer(&mesh_view_constants_array).to_vec(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_WRITE,
            },
        }));

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

        let indirect_buffer_handle = resource_manager.next_buffer();
        engine.send_render_command(RenderCommand::CreateBuffer(CreateBuffer {
            handle: *indirect_buffer_handle,
            buffer_create_info: BufferCreateInfo {
                label: None,
                contents: rs_foundation::cast_to_raw_buffer(&draw_indexed_indirect_args_array)
                    .to_vec(),
                usage: wgpu::BufferUsages::INDIRECT,
            },
        }));

        let mut draw_object = DrawObject::new(
            0,
            vec![*vertex_buffer_handle],
            vertices.len() as u32,
            EPipelineType::Builtin(EBuiltinPipelineType::MeshViewMultipleDraw),
            Some(*index_buffer_handle),
            Some(indices.len() as u32),
            vec![vec![
                EBindingResource::Constants(*self.global_constants_handle),
                EBindingResource::Constants(*constants_handle),
            ]],
        );

        draw_object.draw_call_type = EDrawCallType::MultiDrawIndirect(MultiDrawIndirect {
            indirect_buffer_handle: *indirect_buffer_handle,
            indirect_offset: 0,
            count: REPEAT_SIZE as u32,
        });
        self.draw_objects.clear();
        self.draw_objects.push(MeshViewDrawObject {
            draw_object,
            constants_handle,
            mesh_view_constants_array,
        });
    }
}
