use super::misc::update_window_with_input_mode;
use crate::{
    custom_event::ECustomEventType,
    editor::WindowsManager,
    editor_context::{EWindowType, EditorContext},
    editor_ui,
};
use anyhow::anyhow;
use egui::Sense;
use egui_winit::State;
use glam::Vec4Swizzles;
use rs_core_minimal::primitive_data::PrimitiveData;
use rs_engine::{
    camera::Camera,
    camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle},
    content::particle_system::{EParticleEmiterType, ParticleSpawnEmiterPros},
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    handle::BufferHandle,
    input_mode::EInputMode,
    resource_manager::ResourceManager,
};
use rs_foundation::new::SingleThreadMutType;
use rs_render::{
    command::{
        BufferCreateInfo, CreateBuffer, Draw, DrawObject, EBindingResource, EDrawCallType,
        PresentInfo, RenderCommand, UpdateBuffer,
    },
    scene_viewport::SceneViewport,
    vertex_data_type::mesh_vertex::{Instance0, MeshVertex0},
};
use std::collections::HashMap;
use wgpu::BufferUsages;
use winit::{
    dpi::PhysicalSize,
    event::{MouseButton, MouseScrollDelta, WindowEvent},
};

pub struct DataSource {
    pub particle_system: SingleThreadMutType<rs_engine::content::particle_system::ParticleSystem>,
    pub particle_system_template: rs_engine::particle::system::ParticleSystem,
}

pub struct BaseUIWindow {
    pub egui_winit_state: State,
    pub camera: Camera,
    pub frame_sync: FrameSync,
    pub virtual_key_code_states: HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
    pub global_constants: rs_render::global_uniform::Constants,
    pub global_constants_handle: rs_engine::handle::BufferHandle,
    pub grid_draw_object: DrawObject,
    pub camera_movement_speed: f32,
    pub camera_motion_speed: f32,
    pub input_mode: EInputMode,
}

impl BaseUIWindow {
    pub fn new(
        window_context: &mut crate::editor::WindowContext,
        context: egui::Context,
        engine: &mut Engine,
    ) -> anyhow::Result<BaseUIWindow> {
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
        let input_mode = EInputMode::UI;
        update_window_with_input_mode(window, input_mode);
        Ok(BaseUIWindow {
            egui_winit_state,
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

    pub fn device_event_process(&mut self, device_event: &winit::event::DeviceEvent) {
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

    pub fn window_event_process(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        engine: &mut Engine,
    ) {
        let _ = self.egui_winit_state.on_window_event(window, event);

        match event {
            WindowEvent::Resized(size) => {
                engine.resize(window_id, size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                engine.remove_window(window_id);
            }
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
                engine.recv_output_hook();

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
            }
            _ => {}
        }
    }
}

pub struct ParticleSystemUIWindow {
    pub data_source: DataSource,
    pub context: egui::Context,
    base_ui_window: BaseUIWindow,
    vertex_buffer_handle: BufferHandle,
    index_buffer_handle: BufferHandle,
}

impl ParticleSystemUIWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<ECustomEventType>,
        engine: &mut Engine,
        particle_system: SingleThreadMutType<rs_engine::content::particle_system::ParticleSystem>,
    ) -> anyhow::Result<ParticleSystemUIWindow> {
        let window_context =
            window_manager.spwan_new_window(EWindowType::Particle, event_loop_window_target)?;

        let particle_system_template = particle_system.borrow().new_template_instance();
        let data_source = DataSource {
            particle_system,
            particle_system_template,
        };
        let base_ui_window = BaseUIWindow::new(window_context, context.clone(), engine)?;

        let rm = ResourceManager::default();
        let vertex_buffer_handle = rm.next_buffer();
        let index_buffer_handle = rm.next_buffer();

        let quad = PrimitiveData::quad();

        let command = rs_render::command::RenderCommand::CreateBuffer(CreateBuffer {
            handle: *vertex_buffer_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some(format!("VertexBuffer")),
                contents: rs_foundation::cast_to_raw_buffer(
                    &quad
                        .into_iter()
                        .map(|x| MeshVertex0 {
                            position: (glam::Mat4::from_rotation_x(90_f32.to_radians())
                                * glam::vec4(x.1.x, x.1.y, x.1.z, 1.0))
                            .xyz(),
                            tex_coord: *x.5,
                        })
                        .collect::<Vec<MeshVertex0>>(),
                )
                .to_vec(),
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            },
        });
        engine.send_render_command(command);

        let command = rs_render::command::RenderCommand::CreateBuffer(CreateBuffer {
            handle: *index_buffer_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some(format!("IndexBuffer")),
                contents: rs_foundation::cast_to_raw_buffer(&quad.indices).to_vec(),
                usage: BufferUsages::INDEX,
            },
        });
        engine.send_render_command(command);

        Ok(ParticleSystemUIWindow {
            data_source,
            context,
            base_ui_window,
            vertex_buffer_handle,
            index_buffer_handle,
        })
    }

    pub fn device_event_process(&mut self, device_event: &winit::event::DeviceEvent) {
        self.base_ui_window.device_event_process(device_event);
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
        self.base_ui_window
            .window_event_process(window_id, window, event, engine);
        let window_inner_size = window.inner_size();

        let _ = event_loop_window_target;
        match event {
            WindowEvent::CloseRequested => {
                window_manager.remove_window(EWindowType::Particle);
            }
            WindowEvent::RedrawRequested => {
                let gui_render_output = (|| {
                    let egui_winit_state = &mut self.base_ui_window.egui_winit_state;
                    {
                        let ctx = egui_winit_state.egui_ctx().clone();
                        let viewport_id = egui_winit_state.egui_input().viewport_id;
                        let viewport_info: &mut egui::ViewportInfo = egui_winit_state
                            .egui_input_mut()
                            .viewports
                            .get_mut(&viewport_id)
                            .unwrap();
                        egui_winit::update_viewport_info(viewport_info, &ctx, window, true);
                    }

                    let new_input = egui_winit_state.take_egui_input(window);

                    egui_winit_state.egui_ctx().begin_frame(new_input);
                    egui_winit_state.egui_ctx().clear_animations();

                    let event = ParticleSystemView::draw(
                        window_inner_size,
                        &egui_winit_state.egui_ctx(),
                        &mut self.data_source,
                    );
                    match event {
                        Some(event) => {
                            handle_event(event, &mut self.data_source.particle_system_template);
                        }
                        None => {}
                    }

                    self.data_source.particle_system_template.tick(1.0 / 30.0);

                    let full_output = egui_winit_state.egui_ctx().end_frame();

                    egui_winit_state
                        .handle_platform_output(window, full_output.platform_output.clone());

                    let gui_render_output = rs_render::egui_render::EGUIRenderOutput {
                        textures_delta: full_output.textures_delta,
                        clipped_primitives: egui_winit_state
                            .egui_ctx()
                            .tessellate(full_output.shapes, full_output.pixels_per_point),
                        window_id,
                    };
                    gui_render_output
                })();

                engine.send_render_command(RenderCommand::UiOutput(gui_render_output));

                let quad = PrimitiveData::quad();
                let rm = ResourceManager::default();

                let position_colors: Vec<(glam::Vec3, glam::Vec4)> = (0..self
                    .data_source
                    .particle_system_template
                    .particle_parameters
                    .get_count())
                    .filter_map(|i| {
                        let is_alive = self
                            .data_source
                            .particle_system_template
                            .particle_parameters
                            .is_alive[i];
                        if is_alive {
                            Some((
                                self.data_source
                                    .particle_system_template
                                    .particle_parameters
                                    .positions[i]
                                    .clone(),
                                self.data_source
                                    .particle_system_template
                                    .particle_parameters
                                    .colors[i]
                                    .clone(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                let instances: Vec<Instance0> = position_colors
                    .iter()
                    .map(|(position, color)| Instance0 {
                        position: *position,
                        color: *color,
                    })
                    .collect();

                let instance_buffer_handle = rm.next_buffer();
                let command = rs_render::command::RenderCommand::CreateBuffer(CreateBuffer {
                    handle: *instance_buffer_handle,
                    buffer_create_info: BufferCreateInfo {
                        label: Some(format!("InstanceBuffer")),
                        contents: rs_foundation::cast_to_raw_buffer(&instances).to_vec(),
                        usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
                    },
                });
                engine.send_render_command(command);
                let mut draw_object = DrawObject::new(
                    0,
                    vec![*self.vertex_buffer_handle, *instance_buffer_handle],
                    quad.vertex_positions.len() as u32,
                    rs_render::renderer::PARTICLE_PIPELINE.to_string(),
                    Some(*self.index_buffer_handle),
                    Some(quad.indices.len() as u32),
                    vec![vec![EBindingResource::Constants(
                        *self.base_ui_window.global_constants_handle,
                    )]],
                );
                draw_object.draw_call_type = EDrawCallType::Draw(Draw {
                    instances: 0..(instances.len() as u32),
                });

                engine.send_render_command(RenderCommand::Present(PresentInfo {
                    window_id,
                    draw_objects: vec![draw_object, self.base_ui_window.grid_draw_object.clone()],
                    // draw_objects: vec![draw_object],
                    virtual_texture_pass: None,
                    scene_viewport: SceneViewport::new(),
                }));

                let wait = self
                    .base_ui_window
                    .frame_sync
                    .tick()
                    .unwrap_or(std::time::Duration::from_secs_f32(1.0 / 60.0));
                std::thread::sleep(wait);

                window.request_redraw();
            }
            _ => {}
        }
    }
}

pub enum EEventType {
    CreateEmiter(EParticleEmiterType),
}

pub struct ParticleSystemView {}

impl ParticleSystemView {
    pub fn draw(
        window_inner_size: PhysicalSize<u32>,
        context: &egui::Context,
        data_source: &mut DataSource,
    ) -> Option<EEventType> {
        let mut event = None;
        let _ = window_inner_size;
        let particle_system = data_source.particle_system.clone();
        let particle_system = particle_system.borrow_mut();
        let template = &data_source.particle_system_template;
        let name = particle_system.get_name();

        for (name, emiter) in &template.emiters {
            editor_ui::EditorUI::new_window(
                &format!("{}", name),
                rs_engine::input_mode::EInputMode::UI,
            )
            .open(&mut true)
            .vscroll(true)
            .hscroll(true)
            .resizable(true)
            .show(context, |ui| match emiter {
                rs_engine::particle::emiter::ParticleEmiter::Spawn(emiter) => {
                    ui.label(format!("Rate: {}", emiter.rate));
                    ui.label(format!("Count: {}", emiter.count));
                    ui.label(format!("Time Range: {}", emiter.time_range));
                }
            });
        }

        egui::SidePanel::left("system").show(context, |ui| {
            ui.label(&name);
        });

        egui::Area::new(egui::Id::new("my_area")).show(context, |ui| {
            let response = ui.allocate_response(ui.available_size(), Sense::click());
            response.context_menu(|ui| {
                if ui.button("Create Emiter").clicked() {
                    event = Some(EEventType::CreateEmiter(EParticleEmiterType::Spawn(
                        ParticleSpawnEmiterPros {
                            rate: 1.0,
                            count: 50,
                            time_range: glam::vec2(0.0, 10.0),
                            name: format!("Untitled"),
                        },
                    )));
                    ui.close_menu();
                }
            });
        });

        event
    }
}

fn handle_event(
    event: EEventType,
    particle_system_template: &mut rs_engine::particle::system::ParticleSystem,
) {
    let names = particle_system_template
        .emiters
        .keys()
        .map(|x| x.to_string())
        .collect();
    match event {
        EEventType::CreateEmiter(particle_emiter_type) => match particle_emiter_type {
            EParticleEmiterType::Spawn(particle_spawn_emiter_pros) => {
                let name = EditorContext::make_unique_name(names, particle_spawn_emiter_pros.name);
                particle_system_template.add_emiter(
                    name,
                    rs_engine::particle::emiter::ParticleEmiter::Spawn(
                        rs_engine::particle::emiter::ParticleSpawnEmiter::new(
                            particle_spawn_emiter_pros.rate,
                            particle_spawn_emiter_pros.count,
                            particle_spawn_emiter_pros.time_range,
                        ),
                    ),
                );
            }
        },
    }
}
