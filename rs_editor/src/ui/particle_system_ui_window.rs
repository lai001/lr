use super::misc::update_window_with_input_mode;
use crate::{
    custom_event::ECustomEventType,
    editor_context::EWindowType,
    editor_ui,
    windows_manager::{WindowContext, WindowsManager},
};
use anyhow::anyhow;
use egui::Sense;
use egui_extras::{Column, TableBuilder};
use egui_winit::State;
use rs_core_minimal::name_generator::{self, make_unique_name};
use rs_engine::{
    camera::Camera,
    camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle},
    content::particle_system::{EParticleEmiterType, ParticleSpawnEmiterPros},
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    input_mode::EInputMode,
    particle::emiter_render::EmiterRender,
    resource_manager::ResourceManager,
};
use rs_foundation::new::SingleThreadMutType;
use rs_render::{
    command::{
        BufferCreateInfo, CreateBuffer, DrawObject, PresentInfo, RenderCommand, UpdateBuffer,
    },
    scene_viewport::SceneViewport,
};
use std::collections::HashMap;
use winit::{
    dpi::PhysicalSize,
    event::{MouseButton, MouseScrollDelta, WindowEvent},
};

pub struct DataSource {
    pub particle_system: SingleThreadMutType<rs_engine::content::particle_system::ParticleSystem>,
    pub particle_system_template: rs_engine::particle::system::ParticleSystem,
    pub current_monitor: Option<String>,
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
        window_context: &mut WindowContext,
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
    pub base_ui_window: BaseUIWindow,
    emiter_render: EmiterRender,
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

        let particle_system_template = {
            let particle_system = particle_system.borrow();
            let system_name = particle_system.get_name();
            particle_system.new_template_instance(system_name)
        };
        let data_source = DataSource {
            particle_system,
            particle_system_template,
            current_monitor: None,
        };
        let base_ui_window = BaseUIWindow::new(window_context, context.clone(), engine)?;

        let emiter_render =
            EmiterRender::new(engine, base_ui_window.global_constants_handle.clone());

        Ok(ParticleSystemUIWindow {
            data_source,
            context,
            base_ui_window,

            emiter_render,
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

                    self.data_source.particle_system_template.tick(1.0 / 60.0);

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

                let mut emiter_draw_objects = self
                    .emiter_render
                    .collect_emiter_render(&self.data_source.particle_system_template, engine);
                let mut draw_objects = vec![];

                draw_objects.append(&mut emiter_draw_objects);
                draw_objects.push(self.base_ui_window.grid_draw_object.clone());

                engine.send_render_command(RenderCommand::Present(PresentInfo {
                    window_id,
                    draw_objects,
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
                    ui.label(format!("Rate: {}", emiter.spawn_rate));
                    ui.label(format!("Count: {}", emiter.count_per_spawn));
                    ui.label(format!("Time Range: {}", emiter.time_range));
                }
            });
        }

        egui::SidePanel::left("system").show(context, |ui| {
            ui.label(format!("{} {}", &name, template.time));
            let _ = ui.separator();
            for (name, emiter) in &template.emiters {
                if ui.button(name).clicked() {
                    match emiter {
                        rs_engine::particle::emiter::ParticleEmiter::Spawn(emiter) => {
                            data_source.current_monitor = Some(emiter.name.clone());
                        }
                    }
                }
            }
        });

        if let Some(name) = data_source.current_monitor.as_ref() {
            let emiter = template.emiters.iter().find(|x| x.0 == name);
            if let Some((_, emiter)) = emiter {
                match emiter {
                    rs_engine::particle::emiter::ParticleEmiter::Spawn(emiter) => {
                        Self::monitor(context, emiter);
                    }
                }
            }
        }

        egui::Area::new(egui::Id::new("my_area")).show(context, |ui| {
            let response = ui.allocate_response(ui.available_size(), Sense::click());
            response.context_menu(|ui| {
                if ui.button("Create Emiter").clicked() {
                    let mut name_generator = name_generator::NameGenerator::new(
                        data_source
                            .particle_system_template
                            .emiters
                            .keys()
                            .into_iter()
                            .map(|x| x.clone())
                            .collect(),
                    );
                    let name = name_generator.next("Untitled");
                    event = Some(EEventType::CreateEmiter(EParticleEmiterType::Spawn(
                        ParticleSpawnEmiterPros {
                            rate: 1.0,
                            count: 50,
                            time_range: glam::vec2(0.0, 10.0),
                            name,
                        },
                    )));
                    ui.close_menu();
                }
            });
        });

        event
    }

    fn monitor(context: &egui::Context, emiter: &rs_engine::particle::emiter::ParticleSpawnEmiter) {
        let name = format!("{} Monitor", emiter.name.clone());
        editor_ui::EditorUI::new_window(
            &format!("{}", name),
            rs_engine::input_mode::EInputMode::UI,
        )
        .open(&mut true)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .show(context, |ui| {
            let text_height = egui::TextStyle::Body
                .resolve(ui.style())
                .size
                .max(ui.spacing().interact_size.y);
            let available_height = ui.available_height();
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height);
            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Index");
                    });
                    header.col(|ui| {
                        ui.strong("Lifetime");
                    });
                    header.col(|ui| {
                        ui.strong("Is alive");
                    });
                })
                .body(|body| {
                    body.rows(
                        text_height,
                        emiter.particle_parameters.get_count(),
                        |mut row| {
                            let row_index = row.index();

                            row.col(|ui| {
                                ui.label(row_index.to_string());
                            });

                            let lifetime = emiter.particle_parameters.lifetimes[row_index];
                            row.col(|ui| {
                                ui.label(lifetime.to_string());
                            });

                            let is_alive = emiter.particle_parameters.is_alive[row_index];
                            row.col(|ui| {
                                ui.label(is_alive.to_string());
                            });
                        },
                    );
                });
        });
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
                let name = make_unique_name(names, particle_spawn_emiter_pros.name);
                particle_system_template.add_emiter(
                    rs_engine::particle::emiter::ParticleEmiter::Spawn(
                        rs_engine::particle::emiter::ParticleSpawnEmiter::new(
                            name,
                            particle_spawn_emiter_pros.rate,
                            particle_spawn_emiter_pros.count,
                            particle_spawn_emiter_pros.time_range,
                            1000,
                            glam::vec3(0.0, 0.0, 0.0),
                        ),
                    ),
                );
            }
        },
    }
}
