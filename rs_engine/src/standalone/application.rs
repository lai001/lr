#[cfg(feature = "plugin_shared_crate")]
use crate::plugin::plugin_crate::Plugin;
use crate::{
    content::{content_file_type::EContentFileType, level::Level},
    engine::Engine,
    input_mode::EInputMode,
    player_viewport::PlayerViewport,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
#[cfg(feature = "network")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "network")]
use std::collections::HashMap;

#[cfg(feature = "network")]
#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum ReplicatedFieldType {
    Level,
    NetworkReplicated,
}

pub struct Application {
    _window_id: isize,
    player_view_port: PlayerViewport,
    current_active_level: SingleThreadMutType<Level>,
    _contents: Vec<EContentFileType>,
    #[cfg(feature = "plugin_shared_crate")]
    plugins: SingleThreadMutType<Vec<Box<dyn Plugin>>>,
    #[cfg(feature = "network")]
    pub is_authority: bool,
    #[cfg(feature = "network")]
    pub server: Option<rs_network::server::Server>,
    #[cfg(feature = "network")]
    pub client: Option<rs_network::client::Client>,
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
        // let resource_manager = ResourceManager::default();

        // let global_sampler_handle = resource_manager.next_sampler();
        // let command = RenderCommand::CreateSampler(CreateSampler {
        //     handle: *global_sampler_handle,
        //     sampler_descriptor: wgpu::SamplerDescriptor::default(),
        // });
        // engine.send_render_command(command);

        // let infos = engine.get_virtual_texture_source_infos();
        let mut player_view_port = PlayerViewport::from_window_surface(
            window_id, width, height, // global_sampler_handle,
            engine, // infos,
            input_mode, false,
        );
        let mut current_active_level =
            current_active_level.make_copy_for_standalone(engine, &contents, &mut player_view_port);

        current_active_level.initialize(engine, &contents, &mut player_view_port);
        current_active_level.set_physics_simulate(true);

        #[cfg(feature = "plugin_shared_crate")]
        for plugin in plugins.iter_mut() {
            plugin.on_init(
                engine,
                &mut current_active_level,
                &mut player_view_port,
                &contents,
            );
        }

        Application {
            _window_id: window_id,
            player_view_port,
            #[cfg(feature = "plugin_shared_crate")]
            plugins: SingleThreadMut::new(plugins),
            current_active_level: SingleThreadMut::new(current_active_level),
            _contents: contents,
            #[cfg(feature = "network")]
            is_authority: false,
            #[cfg(feature = "network")]
            server: None,
            #[cfg(feature = "network")]
            client: None,
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent) {
        self.player_view_port.on_device_event(device_event);
        #[cfg(feature = "plugin_shared_crate")]
        {
            let mut plugins = self.plugins.borrow_mut();
            for plugin in plugins.iter_mut() {
                plugin.on_device_event(device_event);
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn on_window_input(
        &mut self,
        window: &mut winit::window::Window,
        ty: crate::input_type::EInputType,
    ) -> Vec<winit::keyboard::KeyCode> {
        let _ = window;
        self.player_view_port.on_window_input(ty.clone());
        #[cfg(feature = "plugin_shared_crate")]
        let mut consume = vec![];
        #[cfg(not(feature = "plugin_shared_crate"))]
        let consume = vec![];
        #[cfg(feature = "plugin_shared_crate")]
        {
            let mut plugins = self.plugins.borrow_mut();
            for plugin in plugins.iter_mut() {
                let mut plugin_consume = plugin.on_window_input(window, ty.clone());
                consume.append(&mut plugin_consume);
            }
        }
        consume
    }

    pub fn on_redraw_requested(
        &mut self,
        engine: &mut Engine,
        ctx: egui::Context,
        #[cfg(not(target_os = "android"))] window: &mut winit::window::Window,
        #[cfg(not(target_os = "android"))] virtual_key_code_states: &std::collections::HashMap<
            winit::keyboard::KeyCode,
            winit::event::ElementState,
        >,
    ) {
        let _ = ctx;
        #[cfg(not(target_os = "android"))]
        let _ = window;

        #[cfg(feature = "network")]
        {
            if self.is_authority {
                if let Some(server) = &mut self.server {
                    server.process_incoming();
                }
            } else {
                if let Some(client) = &mut self.client {
                    for message in client.take_messages() {
                        let data = Self::deserialize_data(&message.data);
                        for (k, v) in data {
                            match k {
                                ReplicatedFieldType::Level => {
                                    let level =
                                        rs_artifact::bincode_legacy::deserialize::<Level>(&v, None);
                                    if let Ok(mut remote_level) = level {
                                        log::trace!(
                                            "To remote level: {}",
                                            &remote_level.get_name()
                                        );
                                        remote_level.initialize(
                                            engine,
                                            &self._contents,
                                            &mut self.player_view_port,
                                        );
                                        self.current_active_level =
                                            SingleThreadMut::new(remote_level);
                                    }
                                }
                                ReplicatedFieldType::NetworkReplicated => {
                                    let replicated_data = Self::deserialize_replicated_data(&v);
                                    let mut active_level = self.current_active_level.borrow_mut();
                                    active_level.visit_network_replicated_mut(
                                        &mut |network_replicated| {
                                            let id = network_replicated.get_network_id();
                                            if let Some(data) = replicated_data.get(id) {
                                                log::trace!(
                                                    "On sync, id: {id}, name: {:?}",
                                                    network_replicated.debug_name()
                                                );
                                                network_replicated.on_sync(data);
                                            }
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "android"))]
        self.player_view_port
            .on_window_input(crate::input_type::EInputType::KeyboardInput(
                virtual_key_code_states,
            ));

        #[cfg(feature = "plugin_shared_crate")]
        {
            let plugins = self.plugins.clone();
            let mut plugins = plugins.borrow_mut();
            for plugin in plugins.iter_mut() {
                plugin.tick(engine, ctx.clone(), &self._contents.clone(), self);
            }
        }
        let mut active_level = self.current_active_level.borrow_mut();
        #[cfg(feature = "network")]
        {
            if self.is_authority {
                if let Some(server) = self.server.as_mut() {
                    let mut network_replicated_data_map = HashMap::new();
                    active_level.visit_network_replicated_mut(&mut |network_replicated| {
                        let data = network_replicated.on_replicated();
                        if !data.is_empty() {
                            let id = network_replicated.get_network_id();
                            network_replicated_data_map.insert(id.to_owned(), data);
                        }
                    });
                    let data = Self::serialize_replicated_data(&network_replicated_data_map);
                    if !data.is_empty() {
                        server.broadcast(&data);
                    }
                }
            }
        }

        self.player_view_port.update_global_constants(engine);

        if let Some(physics) = active_level.get_physics_mut() {
            physics.collision_events.clear();
        }
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

    #[cfg(feature = "network")]
    pub fn open_server_level(&mut self) -> Option<()> {
        if !self.is_authority {
            return None;
        }
        let Some(server) = &mut self.server else {
            return None;
        };
        let current_active_level = self.current_active_level.borrow();
        let level_data =
            rs_artifact::bincode_legacy::serialize(&*current_active_level, None).ok()?;
        let net_data = HashMap::from([(ReplicatedFieldType::Level, level_data)]);
        let net_data = rs_artifact::bincode_legacy::serialize(&net_data, None).ok()?;
        server.broadcast(&net_data);
        return Some(());
    }

    #[cfg(feature = "network")]
    fn deserialize_data(data: &[u8]) -> HashMap<ReplicatedFieldType, Vec<u8>> {
        if data.is_empty() {
            return HashMap::new();
        }
        match rs_artifact::bincode_legacy::deserialize::<HashMap<ReplicatedFieldType, Vec<u8>>>(
            data, None,
        ) {
            Ok(data) => data,
            Err(err) => {
                log::warn!("Deserialize data, {err}, {}", data.len());
                HashMap::new()
            }
        }
    }

    #[cfg(feature = "network")]
    fn serialize_replicated_data(
        network_replicated_data_map: &HashMap<uuid::Uuid, Vec<u8>>,
    ) -> Vec<u8> {
        if network_replicated_data_map.is_empty() {
            return vec![];
        }

        let v = match rs_artifact::bincode_legacy::serialize(network_replicated_data_map, None) {
            Ok(v) => v,
            Err(err) => {
                log::warn!("Serialize replicated data, {err}");
                return vec![];
            }
        };

        let data = HashMap::from([(ReplicatedFieldType::NetworkReplicated, v)]);
        match rs_artifact::bincode_legacy::serialize(&data, None) {
            Ok(data) => data,
            Err(err) => {
                log::warn!("Serialize replicated data, {err}, {}", data.len());
                vec![]
            }
        }
    }

    #[cfg(feature = "network")]
    fn deserialize_replicated_data(data: &[u8]) -> HashMap<uuid::Uuid, Vec<u8>> {
        if data.is_empty() {
            return HashMap::new();
        }
        match rs_artifact::bincode_legacy::deserialize::<HashMap<uuid::Uuid, Vec<u8>>>(data, None) {
            Ok(data) => data,
            Err(err) => {
                log::warn!("Deserialize replicated data, {err}, {}", data.len());
                HashMap::new()
            }
        }
    }

    #[cfg(feature = "plugin_shared_crate")]
    pub fn reload_plugins(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        *self.plugins.borrow_mut() = plugins;
    }

    pub fn current_active_level(&self) -> SingleThreadMutType<Level> {
        self.current_active_level.clone()
    }

    pub fn player_view_port_mut(&mut self) -> &mut PlayerViewport {
        &mut self.player_view_port
    }
}
