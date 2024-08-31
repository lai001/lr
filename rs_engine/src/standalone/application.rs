#[cfg(not(target_os = "android"))]
use crate::camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle};
#[cfg(feature = "plugin_shared_crate")]
use crate::plugin::plugin_crate::Plugin;
use crate::{
    content::{content_file_type::EContentFileType, level::Level},
    engine::Engine,
    input_mode::EInputMode,
    player_viewport::PlayerViewport,
    resource_manager::ResourceManager,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_render::command::{CreateSampler, RenderCommand};

pub struct Application {
    _window_id: isize,
    player_view_port: PlayerViewport,
    #[cfg(feature = "plugin_shared_crate")]
    plugins: Vec<Box<dyn Plugin>>,
    current_active_level: SingleThreadMutType<Level>,
    _contents: Vec<EContentFileType>,
    input_mode: EInputMode,
    camera_movement_speed: f32,
    camera_motion_speed: f32,
}

impl Application {
    pub fn new(
        window_id: isize,
        width: u32,
        height: u32,
        engine: &mut Engine,
        mut current_active_level: Level,
        #[cfg(feature = "plugin_shared_crate")] mut plugins: Vec<Box<dyn Plugin>>,
        contents: Vec<EContentFileType>,
        input_mode: EInputMode,
    ) -> Application {
        let resource_manager = ResourceManager::default();

        let global_sampler_handle = resource_manager.next_sampler();
        let command = RenderCommand::CreateSampler(CreateSampler {
            handle: *global_sampler_handle,
            sampler_descriptor: wgpu::SamplerDescriptor::default(),
        });
        engine.send_render_command(command);

        Self::add_new_actors(engine, current_active_level.actors.to_vec(), &contents);
        current_active_level.initialize(engine);
        current_active_level.set_physics_simulate(true);

        let infos = engine.get_virtual_texture_source_infos();
        let player_view_port = PlayerViewport::new(
            window_id,
            width,
            height,
            global_sampler_handle,
            engine,
            infos,
        );

        #[cfg(feature = "plugin_shared_crate")]
        for plugin in plugins.iter_mut() {
            plugin.on_init(engine, &mut current_active_level);
        }

        Application {
            _window_id: window_id,
            player_view_port,
            #[cfg(feature = "plugin_shared_crate")]
            plugins,
            current_active_level: SingleThreadMut::new(current_active_level),
            _contents: contents,
            input_mode,
            camera_movement_speed: 0.1,
            camera_motion_speed: 0.1,
        }
    }

    fn add_new_actors(
        engine: &mut Engine,
        actors: Vec<SingleThreadMutType<crate::actor::Actor>>,
        files: &[EContentFileType],
    ) {
        for actor in actors {
            let actor = actor.borrow_mut();
            let mut root_scene_node = actor.scene_node.borrow_mut();
            match &mut root_scene_node.component {
                crate::scene_node::EComponentType::SceneComponent(_) => todo!(),
                crate::scene_node::EComponentType::StaticMeshComponent(static_mesh_component) => {
                    let mut static_mesh_component = static_mesh_component.borrow_mut();
                    static_mesh_component.initialize(ResourceManager::default(), engine, files);
                }
                crate::scene_node::EComponentType::SkeletonMeshComponent(
                    skeleton_mesh_component,
                ) => {
                    skeleton_mesh_component.borrow_mut().initialize(
                        ResourceManager::default(),
                        engine,
                        files,
                    );
                }
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn on_input(&mut self, ty: crate::input_type::EInputType) {
        use crate::input_type::EInputType;
        match ty {
            EInputType::Device(device_event) => {
                //
                match device_event {
                    winit::event::DeviceEvent::MouseMotion { delta } => {
                        DefaultCameraInputEventHandle::mouse_motion_handle(
                            &mut self.player_view_port.camera,
                            *delta,
                            self.input_mode,
                            self.camera_motion_speed,
                        );
                    }
                    _ => {}
                }
            }
            EInputType::MouseWheel(_) => {}
            EInputType::MouseInput(_, _) => {}
            EInputType::KeyboardInput(_) => {}
        }
        #[cfg(feature = "plugin_shared_crate")]
        for plugin in self.plugins.iter_mut() {
            plugin.on_input(ty.clone());
        }
    }

    pub fn on_redraw_requested(
        &mut self,
        engine: &mut Engine,
        ctx: egui::Context,
        #[cfg(not(target_os = "android"))] virtual_key_code_states: &std::collections::HashMap<
            winit::keyboard::KeyCode,
            winit::event::ElementState,
        >,
    ) {
        let _ = ctx;
        let _ = self.input_mode;
        let _ = self.camera_movement_speed;
        let _ = self.camera_motion_speed;

        let mut active_level = self.current_active_level.borrow_mut();

        #[cfg(not(target_os = "android"))]
        for (virtual_key_code, element_state) in virtual_key_code_states {
            DefaultCameraInputEventHandle::keyboard_input_handle(
                &mut self.player_view_port.camera,
                virtual_key_code,
                element_state,
                self.input_mode,
                self.camera_movement_speed,
            );
        }

        #[cfg(feature = "plugin_shared_crate")]
        for plugin in self.plugins.iter_mut() {
            plugin.tick(
                engine,
                &mut active_level,
                ctx.clone(),
                &mut self.player_view_port,
                &self._contents,
            );
        }

        self.player_view_port.update_global_constants(engine);

        active_level.tick();
        if let Some(light) = active_level.directional_lights.first().cloned() {
            let mut light = light.borrow_mut();
            light.update(engine);
            engine.update_light(&mut light);
        }
        for actor in active_level.actors.clone() {
            let actor = actor.borrow_mut();
            let mut root_scene_node = actor.scene_node.borrow_mut();
            match &mut root_scene_node.component {
                crate::scene_node::EComponentType::SceneComponent(_) => todo!(),
                crate::scene_node::EComponentType::StaticMeshComponent(static_mesh_component) => {
                    let mut static_mesh_component = static_mesh_component.borrow_mut();
                    static_mesh_component.update(
                        engine.get_game_time(),
                        engine,
                        active_level.get_rigid_body_set_mut(),
                    );
                    for draw_object in static_mesh_component.get_draw_objects_mut() {
                        self.player_view_port
                            .update_draw_object(engine, draw_object);
                        self.player_view_port.push_to_draw_list(draw_object);
                    }
                }
                crate::scene_node::EComponentType::SkeletonMeshComponent(
                    skeleton_mesh_component,
                ) => {
                    let mut skeleton_mesh_component = skeleton_mesh_component.borrow_mut();
                    skeleton_mesh_component.update(engine.get_game_time(), engine);

                    for draw_object in skeleton_mesh_component.get_draw_objects_mut() {
                        self.player_view_port
                            .update_draw_object(engine, draw_object);
                        self.player_view_port.push_to_draw_list(draw_object);
                    }
                }
            }
        }
        if let Some(physics) = active_level.get_physics_mut() {
            self.player_view_port.physics_debug(
                engine,
                &physics.rigid_body_set,
                &physics.collider_set,
            );
        }
        engine.present_player_viewport(&mut self.player_view_port);
    }

    pub fn on_size_changed(&mut self, width: u32, height: u32) {
        self.player_view_port.camera.set_window_size(width, height);
    }

    #[cfg(feature = "plugin_shared_crate")]
    pub fn reload_plugins(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        self.plugins = plugins;
    }
}
