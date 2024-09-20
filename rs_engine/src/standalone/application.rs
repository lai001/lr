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
    current_active_level: SingleThreadMutType<Level>,
    _contents: Vec<EContentFileType>,
    #[cfg(feature = "plugin_shared_crate")]
    plugins: Vec<Box<dyn Plugin>>,
}

impl Application {
    pub fn new(
        window_id: isize,
        width: u32,
        height: u32,
        engine: &mut Engine,
        current_active_level: &Level,
        contents: Vec<EContentFileType>,
        input_mode: EInputMode,
        #[cfg(feature = "plugin_shared_crate")] mut plugins: Vec<Box<dyn Plugin>>,
    ) -> Application {
        let resource_manager = ResourceManager::default();

        let global_sampler_handle = resource_manager.next_sampler();
        let command = RenderCommand::CreateSampler(CreateSampler {
            handle: *global_sampler_handle,
            sampler_descriptor: wgpu::SamplerDescriptor::default(),
        });
        engine.send_render_command(command);

        let infos = engine.get_virtual_texture_source_infos();
        let mut player_view_port = PlayerViewport::new(
            window_id,
            width,
            height,
            global_sampler_handle,
            engine,
            infos,
            input_mode,
            false,
        );
        let mut current_active_level =
            current_active_level.make_copy_for_standalone(engine, &contents, &mut player_view_port);

        current_active_level.initialize(engine, &contents, &mut player_view_port);
        current_active_level.set_physics_simulate(true);

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
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn on_input(&mut self, ty: crate::input_type::EInputType) {
        self.player_view_port.on_input(ty.clone());

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

        let mut active_level = self.current_active_level.borrow_mut();

        #[cfg(not(target_os = "android"))]
        self.player_view_port
            .on_input(crate::input_type::EInputType::KeyboardInput(
                virtual_key_code_states,
            ));

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

        active_level.tick(engine.get_game_time(), engine, &mut self.player_view_port);
        let mut draw_objects = active_level.collect_draw_objects();
        for draw_object in draw_objects.iter_mut() {
            self.player_view_port
                .update_draw_object(engine, draw_object);
            draw_object.switch_player_viewport(&self.player_view_port);
        }
        self.player_view_port.append_to_draw_list(&draw_objects);

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
